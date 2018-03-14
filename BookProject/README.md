# Installation and Usage (without IntelliJ)

  1. Install `sbt` server.

    ```
    brew install sbt
    ```

  2. Compile and assemble into fat jar. Make sure to run this command at the root of the Scala project directory.

    ```
    sbt compile assembly
    ```

  3. Run jar file.

    ```
    java -jar target/scala-2.11/gutenberg-analysis-assembly-0.1.jar -t ../data/texts/ -o ../data/ -s ../data/stopwords.txt -r 500
    ```
