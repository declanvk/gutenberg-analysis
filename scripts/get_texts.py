#!/usr/bin/env python

from multiprocessing import Pool
from functools import partial
from utils.acquire import load_etext
from utils.cleanup import strip_headers
import sys
import os
from pathlib import Path
import csv
import progressbar

def job(output_directory, text_id):
    text = load_etext(text_id)
    clean_text = strip_headers(text)

    output_path = output_directory / "{}.txt".format(text_id)
    with open(output_path, 'w') as output_file:
        output_file.write(clean_text)

def run_all_jobs(output_directory, text_ids, chunk_size):
    job_instance = partial(job, output_directory)

    progress_bar = progressbar.ProgressBar(max_value=len(text_ids))
    counter = 0
    def update_progress_bar(output):
        nonlocal counter
        counter += 1
        progress_bar.update(counter)

    with Pool() as pool:
        result = pool.map_async(job_instance, text_ids, chunk_size, update_progress_bar)

        result.wait()

def load_ids(text_ids_path):
    with open(text_ids_path, 'r') as text_ids_file:
        has_header = csv.Sniffer().has_header(text_ids_file.read(1024))
        text_ids_file.seek(0)  # Rewind.

        text_ids_reader = csv.reader(text_ids_file)
        if has_header:
            next(text_ids_reader)  # Skip header row.

        for row in text_ids_reader:
            yield int(row[0])

def main():
    argc = len(sys.argv)
    if argc != 3:
        raise ValueError(f"Illegal number of arguments passed, {argc}")

    eligible_texts_file = Path.cwd() / Path(sys.argv[1])
    output_path = Path.cwd() / Path(sys.argv[2])

    if not eligible_texts_file.exists():
        raise FileNotFoundError("Input file does not exist")

    if not output_path.exists():
        os.makedirs(output_path, exist_ok=True)

    text_ids = list(load_ids(eligible_texts_file))
    run_all_jobs(output_path, text_ids, CHUNK_SIZE)

CHUNK_SIZE = 20

if __name__ == '__main__':
    try:
        main()
    except (ValueError, FileNotFoundError) as e:
        print(e, file=sys.stderr)
        print("usage: ./get_texts.py <eligible-texts-file> <output-directory>", file=sys.stderr)