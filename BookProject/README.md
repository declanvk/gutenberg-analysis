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

**LOCALLY**
```
spark-submit --class main.App --master local[*] target/scala-2.11/gutenberg-processing-assembly-0.1.jar \
    -l ../data/texts_listing.txt
    -t ../data/texts/ \
    -o ../data/spark/ \
    -s ../data/stopwords.txt \
    -r 300 \
    -m Local
```

**ON CLUSTER**
```
export AWS_ACCESS_KEY_ID=<....>
export AWS_SECRET_ACCESS_KEY=<....>

spark-submit --class main.App --master local[*] target/scala-2.11/gutenberg-processing-assembly-0.1.jar \
    -l "s3a://english-gutenberg-texts/texts_listing.txt" \
    -t s3a://english-gutenberg-texts/texts/ \
    -o ../data/spark/ \
    -s s3a://english-gutenberg-texts/stopwords.txt \
    -r 30 \
    -m S3
```