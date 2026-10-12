#!/usr/bin/env python3
"""Bounded projection of existing pytest JUnit XML received on stdin."""

from __future__ import annotations

import argparse
import json
import sys
import xml.etree.ElementTree as ET


IDENTITY_CHARS = 512
MESSAGE_CHARS = 512


class UnsafeXmlDeclaration(ValueError):
    """Raised before a DTD can define or expand an entity."""


class RejectingTreeBuilder(ET.TreeBuilder):
    def doctype(self, name: str, pubid: str | None, system: str | None) -> None:
        del name, pubid, system
        raise UnsafeXmlDeclaration("DTD declarations are not supported")


def bounded(value: object, maximum: int) -> str:
    text = str(value or "").replace("\x00", "").strip()
    if len(text) <= maximum:
        return text
    return f"{text[: maximum - 15]}...[truncated]"


def local_name(tag: str) -> str:
    return tag.rsplit("}", 1)[-1]


def child(testcase: ET.Element, name: str) -> ET.Element | None:
    return next((item for item in testcase if local_name(item.tag) == name), None)


def identity(testcase: ET.Element) -> str:
    name = bounded(testcase.attrib.get("name", "<unnamed>"), IDENTITY_CHARS)
    owner = testcase.attrib.get("classname") or testcase.attrib.get("file") or ""
    if owner:
        return bounded(f"{owner}::{name}", IDENTITY_CHARS)
    return name


def failure_message(item: ET.Element) -> str:
    return bounded(item.attrib.get("message") or item.text or "", MESSAGE_CHARS)


def fail(error_code: str, message: str) -> None:
    print(json.dumps({"ok": False, "errorCode": error_code, "message": message}))


def main() -> int:
    parser = argparse.ArgumentParser(add_help=False)
    parser.add_argument("--max-failures", type=int, required=True)
    arguments = parser.parse_args()
    if arguments.max_failures < 1 or arguments.max_failures > 25:
        fail("invalid_arguments", "maxFailures is outside the parser bound.")
        return 0

    source = sys.stdin.buffer.read()
    declarations = source.upper()
    if b"<!DOCTYPE" in declarations or b"<!ENTITY" in declarations:
        fail("unsafe_xml", "JUnit XML declarations and entities are not supported.")
        return 0
    try:
        parser_target = RejectingTreeBuilder()
        root = ET.fromstring(source, parser=ET.XMLParser(target=parser_target))
    except UnsafeXmlDeclaration:
        fail("unsafe_xml", "JUnit XML declarations and entities are not supported.")
        return 0
    except (ET.ParseError, ValueError, UnicodeError):
        fail("malformed_xml", "The JUnit XML report is malformed.")
        return 0

    if local_name(root.tag) not in {"testsuite", "testsuites"}:
        fail("not_junit_xml", "The XML root must be testsuite or testsuites.")
        return 0

    counts = {"total": 0, "passed": 0, "failed": 0, "errors": 0, "skipped": 0}
    failures: list[dict[str, str]] = []
    observed_failures = 0
    for testcase in (element for element in root.iter() if local_name(element.tag) == "testcase"):
        counts["total"] += 1
        error = child(testcase, "error")
        failure = child(testcase, "failure")
        skipped = child(testcase, "skipped")
        if error is not None:
            counts["errors"] += 1
            observed_failures += 1
            if len(failures) < arguments.max_failures:
                failures.append(
                    {"identity": identity(testcase), "kind": "error", "message": failure_message(error)}
                )
        elif failure is not None:
            counts["failed"] += 1
            observed_failures += 1
            if len(failures) < arguments.max_failures:
                failures.append(
                    {"identity": identity(testcase), "kind": "failure", "message": failure_message(failure)}
                )
        elif skipped is not None:
            counts["skipped"] += 1
        else:
            counts["passed"] += 1

    print(
        json.dumps(
            {
                "ok": True,
                "counts": counts,
                "failures": failures,
                "failuresTruncated": observed_failures > len(failures),
            },
            ensure_ascii=False,
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
