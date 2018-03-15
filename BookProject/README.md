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

```
spark-submit --class App --master local[*] target/scala-2.11/gutenberg-processing-assembly-0.1.jar -t ../data/texts/ -o ../data/spark/ -s ../data/stopwords.txt -r 300
```
