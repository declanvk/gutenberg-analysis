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

Run these commands from the top level directory of the project.

```
./scripts/download_catalog data/catalog/raw

./scripts/collate_query_results.py data/catalog/raw data/catalog/full.csv ./scripts/sparql/get_all_attributes_optional.rq

./scripts/extract_eligible_texts.py data/catalog/full.csv data/eligible_works.csv data/labels.csv

./scripts/get_texts.py data/eligible_works.csv data/texts
```

**WARNING** - This script will run for several hours. Make sure you'll be uninterrupted for an extended amount of time.

## Data (`data/`)

The data directory contains some pieces of permanent data, but is mostly filled with intermediate results that are never commited to git.

## Documentation (`Docs/`)

This directory contains the logs for the amount of work each person completed, some reasoning behind our methodology. We explain why we chose Hadoop and Spark, how the internal of our spark programs work, and some results.

* Working time log - [`Docs/log.txt`](Docs/log.txt)
* Explanation of Scala code internals - [`Docs/internals.txt`](Docs/internals.txt)
* Justification for using Hadoop - [`Docs/hadoop-justification.txt`](Docs/hadoop-justification.txt)

## Spark Code (`BookProject/`)

This directory contains our Spark Scala code, the main engine behind the processing our of data. 

## Notebook Analysis (`analysis/`)

Throughtout our project we had to make decisions based on the characteristics of our data. We use Jupyter Notebooks to visualize and experiment with our data at different stages of our process. These notebooks should provide some insight into our methods.

* Analysis of Project Gutenberg data catalog - [`analysis/CatalogAnalysis.ipynb`](analysis/CatalogAnalysis.ipynb)
* Analysis of unique words across all documents - [`analysis/Filtering.ipynb`](analysis/Filtering.ipynb)
* Future experimental results - [`analysis/Results.ipynb`](analysis/Results.ipynb)

## Authors

* **Maxence Weinrich** - [@maxwey](https://github.com/maxwey)
* **Isaac Hinterg** - [@ihinterg](https://github.com/ihinterg)
* **Declan Kelly** - [@declanvk](https://github.com/declanvk)

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE) file for details
