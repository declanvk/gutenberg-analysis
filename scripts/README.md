# Script Usage

## download_catalog

```
./scripts/download_catalog <output-directory>
```

This script is intended to download and extract the catalog to a specific output directory. The script will create all intermediate folders on the path specified, and the output directory if it does not exist.

The unpacked format of the catalog will be
```
<output-directory>
└── cache
    └── epub
        ├── 0
        │   └── pg0.rdf
        ├── 1
        │   └── pg1.rdf
        ├── 10
        │   └── pg10.rdf
        ├── 100
        │   └── pg100.rdf
        ├── 1000
        │   └── pg1000.rdf
        ├── 10000
        │   └── pg10000.rdf
        ├── 10001
        .   └── pg10001.rdf
        .
        .
        .
```

### Example

```
./scripts/download_catalog catalog/raw
```

