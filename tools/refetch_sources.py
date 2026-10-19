"""Reproduce a source snapshot into a separate directory, enforcing pinned hashes."""
import argparse
import hashlib
from html import unescape
from html.parser import HTMLParser
import json
from pathlib import Path
from urllib.parse import urldefrag
from urllib.request import Request, urlopen


class JudicialText(HTMLParser):
    def __init__(self):
        super().__init__()
        self.active = False
        self.parts = []

    def handle_starttag(self, tag, attrs):
        if tag == "noframes":
            self.active = True

    def handle_endtag(self, tag):
        if tag == "noframes":
            self.active = False

    def handle_data(self, data):
        if self.active:
            self.parts.append(data)


def extract(raw: bytes, extractor: str) -> bytes:
    if extractor == "identity":
        return raw
    if extractor != "html_noframes_normalized":
        raise ValueError("unknown extractor")
    parser = JudicialText()
    parser.feed(raw.decode("utf-8"))
    # HTMLParser treats noframes as raw text, so entity decoding is explicit.
    text = " ".join(unescape(" ".join(parser.parts)).split())
    if not text:
        raise ValueError("judicial opinion not found; extraction must be reviewed")
    return (text + "\n").encode("utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("lock", type=Path)
    parser.add_argument("output", type=Path, help="new snapshot directory; existing files are never overwritten")
    args = parser.parse_args()
    lock = json.loads(args.lock.read_text())
    if lock["format"] != "oathkeeper-source-lock/1":
        raise ValueError("unsupported source lock")
    output = args.output.resolve()
    for source in lock["sources"]:
        target = (output / source["file"]).resolve()
        if not target.is_relative_to(output):
            raise ValueError("snapshot path escapes output")
        uri = urldefrag(source["uri"])[0]
        with urlopen(Request(uri, headers={"User-Agent": "Oathkeeper source replay"}), timeout=30) as response:
            raw = response.read()
        data = extract(raw, source["extractor"])
        if hashlib.sha256(data).hexdigest() != source["sha256"]:
            raise ValueError(f"upstream source changed: {source['id']}; create a reviewed new snapshot")
        target.parent.mkdir(parents=True, exist_ok=True)
        with target.open("xb") as stream:
            stream.write(data)
        print(source["id"])


if __name__ == "__main__":
    main()
