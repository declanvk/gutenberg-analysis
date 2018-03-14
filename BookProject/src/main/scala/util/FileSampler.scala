package util

import scala.util._
import java.io.File

object FileSampler {

  // Helper function to get all the files from a directory
  def getListOfFiles(dir: String):List[File] = {
    val d = new File(dir)
    if (d.exists && d.isDirectory) {
      d.listFiles.filter(_.isFile).toList
    } else {
      List[File]()
    }
  }

  // sampleDirectory takes to arguments a directory and the number of random fies from that directory you want returned
  def sampleDirectory(directory:String, n:Int):List[File] = {
    val ret:List[File] = getListOfFiles(directory)

    Random.shuffle(ret).take(n)
  }
}
