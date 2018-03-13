import org.apache.spark.SparkContext._
import scala.io._
import java.io._
import org.apache.spark.{ SparkConf, SparkContext }
import org.apache.spark.rdd._
import org.apache.log4j.Logger
import org.apache.log4j.Level
import scala.collection._
import scala.util.Try

object App {
  def main(args: Array[String]): Unit = {
    Logger.getLogger("org").setLevel(Level.OFF)
    Logger.getLogger("akka").setLevel(Level.OFF)

    val conf = new SparkConf().setAppName("App").setMaster("local[4]")
    val sc = new SparkContext(conf)

    val outfile = new BufferedWriter(new FileWriter(new File("out.txt")))

    val removableWords = sc.textFile("resources/stopwords.txt")
        .flatMap(x => x.split("\\b+")) //words to be removed from the dictionary
        .map(x => x.toLowerCase.trim)


    val catalog = sc.textFile("resources/catalog.txt")
        .map(x => (x.split(",")(0).trim, x.split(",")(1).trim)) //get the name of the book & the genre associated with it

    // Generate the Dictionary
    val dictionary = sc.textFile("../data/texts")
          .flatMap(_.split("\\b+")) //split all words by nonword characters
          .map(_.trim.toLowerCase)  //set all words to lowercase
          .filter(_.matches("[a-zA-Z]{3,}")) // filter out non words of length less than 3
          .subtract(removableWords) // remove all words in removable list
          .countByValue()
          .filter(_._2 >= 3) //only keep words that occur at least three times


    val wordCount = dictionary
          .toList
          .sortWith((x,y) => x._2 > y._2)
          .foreach(x => {
            val str = x.toString()
            outfile.write(str + "\n")
          })

    val orderedKeys = dictionary
        .keys
        .toList
        .sortWith((x,y) => x < y)

    val indexedDict = orderedKeys.map(x => (x, orderedKeys.indexOf(x)))
    indexedDict.foreach(x => {
      val str = x.toString()
      outfile.write(str + "\n")
    })
    val dictRDD = sc.parallelize(indexedDict) //dictionary list => RDD


    // Generate a 'vector' from each book by using the dictionary created previously.
    val bookData = sc.wholeTextFiles("../data/texts") // get all the books into one RDD
          .map(x => {
            val loc = x._1.lastIndexOf('/')+1
            (x._1.substring(loc), x._2)
          }) // map book name and its entire text
          .flatMapValues(x => x.split("\\b+")) //split text into to words, generating a new k/v pair for each word
          .map(x => (x._2.trim.toLowerCase, x._1))  //remove spacings, set to lowercase. Change so word is the key
          .leftOuterJoin(dictRDD)  // join dictionary words and words in text
          .map({
            case (word, (book, Some(index))) =>  ((book, index), 1)
            case (word, (book, None)) => ((book, -1), 0)
          })
          .filter(_._1._2 != -1) // Remove words not in the dictionary
          .groupByKey() // group words together
          .map(x => (x._1._1, (x._1._2, x._2.sum))) // map each entry to key=bookID value=(wordIndex (in dict), timesWordOccurred (in book))
          .groupByKey()  //combine data by each book identifier



    bookData.collect().foreach(x => {
      val str = x.toString()
      outfile.write(str + "\n")
    })

    outfile.close()

  }
}
