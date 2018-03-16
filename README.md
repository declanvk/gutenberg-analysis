# Gutenbery Text Analysis

Using Spark to analyze texts from Project Gutenberg.

## Scripts (`scripts/`)

These scripts will allow you to download and manipulate the data into a usable form for later analysis.

### Prerequisites

It is recommended to use some form of python environment manager such as `conda`.

### Installing

Create a new environment with your python environment manager, specifying Python 3.

```
conda env create -f script/environment.yaml
```

Activate the conda environment.

```
source activate gutenberg
```

### Acquire Data

Run this script from the top level directory of the project.

```
./scripts/run_complete_pipeline
```

**WARNING** - This script will run for several hours. Make sure you'll be uninterrupted for an extended amount of time.

### Data Directory Layout

After running the complete data acquisition pipeline, the `data` directory should look like this:

```
data/
├── catalog/
│   ├── authors.csv
│   ├── full.csv
│   ├── genres.csv
│   ├── raw/
│   │   └── cache/
│   │       └── epub/
│   │           ├── 0
│   │           │   └── pg0.rdf
│   │           ├── 1
│   │           │   └── pg1.rdf
│   │           ├── 10
│   │           │   └── pg10.rdf
│   │           ├── 100
│   │           │   └── pg100.rdf
│   │           ├── 1000
│   │           │   └── pg1000.rdf
│   │           ├── 10000
│   │           │   └── pg10000.rdf
.	.			.
.	.			.
.	.			.
│   │           ├── 9996
│   │           │   └── pg9996.rdf
│   │           ├── 9997
│   │           │   └── pg9997.rdf
│   │           ├── 9998
│   │           │   └── pg9998.rdf
│   │           ├── 9999
│   │           │   └── pg9999.rdf
│   │           ├── 999999
│   │           │   └── pg999999.rdf
│   │           └── DELETE-55495
│   │               └── pg55485.rdf
│   └── works.csv
├── eligible_works.csv
└── texts/
    ├── 1.txt
    ├── 10.txt
    ├── 100.txt
    ├── 10000.txt
    ├── 10001.txt
	.
	.
	.
    ├── 9995.txt
    ├── 9996.txt
    ├── 9997.txt
    ├── 9998.txt
    └── 9999.txt

```

## Authors

* **Maxence Weinrich** - [@maxwey](https://github.com/maxwey)
* **Isaac Hinterg** - [@ihinterg](https://github.com/ihinterg)
* **Declan Kelly** - [@declanvk](https://github.com/declanvk)

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE) file for details
