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

## collate_query_results.py

```
./scripts/collate_query_results.py <catalog-directory> <output-path> <query-path>
```

This script will aggregate a raw catalog file structure into a single RDF graph, then execute a SPARQL query. The result of the query is saved in CSV format to the specified output path.

### Example

```
./scripts/collate_query_results.py data/catalog/raw data/catalog/full.csv scripts/sparql/get_all_attributes_optional.rq 
```

## extract_eligible_texts.py

```
./scripts/extract_eligible_texts.py <catalog-file> <output-file>
```

This script will take the raw collated catalog file as input and perform some transformations and extractions to determine which texts are eligible for later analysis. The output of this script will be a csv file containing one row for each eligible text, along with book id, title, date of publishing, author, and most frequently occuring subject.

### Example

```
./scripts/extract_eligible_texts.py data/catalog/full.csv data/eligible_works.csv
```

## get_texts.py

```
./scripts/get_texts.py <eligible-texts-file> <output-directory>
```

This script will take a csv file with the format:

```
book,title,published,subject,subject_freq
1,The Declaration of Independence of the United States of America,1971-12-01,History,0.026040938940897023
2,The United States Bill of Rights: The Ten Original Amendments to the Constitution of the United States,1972-12-01,United States,0.011889583624423642
3,John F. Kennedy's Inaugural Address,1973-11-01,United States,0.011889583624423642
4,"Lincoln's Gettysburg Address: Given November 19, 1863 on the battlefield near Gettysburg, Pennsylvania, USA",1973-11-01,E456,0.001790205393321224
5,The United States Constitution,1975-12-01,United States,0.011889583624423642
6,Give Me Liberty or Give Me Death,1976-12-01,United States,0.011889583624423642
7,The Mayflower Compact,1977-12-01,History,0.026040938940897023
8,Abraham Lincoln's Second Inaugural Address,1978-12-01,United States,0.011889583624423642
9,Abraham Lincoln's First Inaugural Address,1979-12-01,United States,0.011889583624423642
```

and will use the the first column of book ids to download texts from a mirror of Project Gutenberg. The script will download each text, then remove the header and footer that contains text irrelevant to the work. Each cleaned text will be saved as an individual file under the specified output directory, using the book id as the name of the file.

### Example

```
./scripts/get_texts.py data/eligible_works.txt data/texts
```

## run_complete_pipeline

```
./scripts/run_complete_pipeline
```

This script will run all the above script in the given order, output each intermediate result to standardized locations. Make sure to run the script from the top level of the project directory, so that the `data` directory does not get placed inside the `script` directory.

### Example

```
./scripts/run_complete_pipeline
```