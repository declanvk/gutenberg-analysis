#!/usr/bin/env python

from __future__ import print_function
import sys

from gutenberg.acquire import set_metadata_cache
from gutenberg.acquire.metadata import SqliteMetadataCache

def main():
    argc = len(sys.argv)
    if argc != 2:
        print("usage: ./load_gutenberg_cache.py <sqlite-cache-location>", file=sys.stderr)
        sys.exit(1)

    cache = SqliteMetadataCache(sys.argv[1])
    cache.populate()
    set_metadata_cache(cache)

if __name__ == '__main__':
    main()