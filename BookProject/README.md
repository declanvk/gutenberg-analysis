# Installation and Usage (without IntelliJ)

  1. Install `sbt` server.

```
brew install sbt
```

  2. Compile and assemble into fat jar. Make sure to run this command at the root of the Scala project directory.

```
sbt compile assembly
```

  3. Run jar file using `spark-submit`.

### Locally
```
spark-submit --class main.App --master local[*] target/scala-2.11/gutenberg-processing-assembly-0.1.jar \
    -l ../data/filtered_texts_listing.txt \
    -t ../data/texts/ \
    -o ../data/spark/ \
    -s ../data/stopwords.txt \
    -b ../data/labels.csv \
    -r 200 \
    -m Local \
    -a Dictionary,DocumentVectors,SimilarityMatrix,kNearest,Results
```

### On Generic Cluster
```
export AWS_ACCESS_KEY_ID=<....>
export AWS_SECRET_ACCESS_KEY=<....>

spark-submit --class main.App --master yarn target/scala-2.11/gutenberg-processing-assembly-0.1.jar \
    -l s3a://english-gutenberg-texts/filtered-texts-listing.txt \
    -t s3a://english-gutenberg-texts/texts/ \
    -o s3a://english-gutenberg-texts/output/ \
    -s s3a://english-gutenberg-texts/stopwords.txt \
    -b s3a://english-gutenberg-texts/labels.csv \
    -r 2000 \
    -m S3 \
    -a Dictionary,DocumentVectors,SimilarityMatrix,kNearest,Results
```

### On AWS Elastic Map-Reduce Cluster
```
spark-submit --deploy-mode cluster --class main.App --master yarn \
    --conf spark.hadoop.fs.s3.awsAccessKeyId=<...> --conf spark.hadoop.fs.s3a.access.key=<...> \
    --conf spark.hadoop.fs.s3.awsSecretAccessKey=<...> --conf spark.hadoop.fs.s3a.secret.key=<...> \
    --conf spark.yarn.executor.memoryOverhead=10 \
    s3://english-gutenberg-texts/gutenberg-processing-assembly-0.1.jar \
    -l s3://english-gutenberg-texts/filtered-texts-listing.txt \
    -t s3://english-gutenberg-texts/texts/ \
    -o s3://english-gutenberg-texts/output/ \
    -s s3://english-gutenberg-texts/stopwords.txt \
    -r 2000 \
    -m S3 \
    -a SimilarityMatrix,Results
```
