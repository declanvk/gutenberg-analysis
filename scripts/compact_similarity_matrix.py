#!/usr/bin/env python

import pandas as pd
import numpy as np
from pathlib import Path
import re
import json as js
from functools import cmp_to_key
import sys
import progressbar

SIMILARITY_ENTRY_REG = re.compile("^\((\d+),(\d+),(.*)\)$")

def load_chunk(chunk_path):
    with open(chunk_path, mode='r') as chunk:
        chunk = js.load(chunk)
        chunk.sort()

        row_index = list(set(map(lambda x: x[0], chunk)))
        row_index.sort()
        row_index = pd.Series(row_index)
        row_index = pd.Series(row_index.index.values, index=row_index.values)
        dim_row = len(row_index)
        column_index = list(set(map(lambda x: x[1], chunk)))
        column_index.sort()
        column_index = pd.Series(column_index)
        column_index = pd.Series(column_index.index.values, index=column_index.values)
        dim_column = len(column_index)
        
        matrix = np.empty(dtype='float32', shape=(dim_row, dim_column))
        for (r, c, s) in chunk:
            matrix[row_index[r], column_index[c]] = s
        
        return ((row_index, column_index), matrix.reshape((dim_row, dim_column)))

def load_all_chunks(matrix, chunk_folder):
    progress = progressbar.ProgressBar()
    for path in progress(chunk_folder.iterdir()):
        ((row_index, column_index), chunk) = load_chunk(path)

        row_offset = document_index[row_index.index.min()]
        column_offset = document_index[column_index.index.min()]
        matrix[row_offset:(row_offset + chunk.shape[0]), column_offset:(column_offset + chunk.shape[1])] = chunk

def main():
    argc = len(sys.argv)
    if argc != 3:
        raise ValueError(f"Illegal number of arguments passed, {argc}")

    similarity_chunks_folder = Path.cwd() / Path(sys.argv[1])
    filtered_documents = Path.cwd() / Path(sys.argv[2])
    similarity_matrix_file = Path.cwd() / Path(sys.argv[3])

    if not similarity_chunks_folder.exists():
        raise FileNotFoundError("Input similarity chunks folder does not exist")

    if not filtered_documents.exists():
        raise FileNotFoundError("Input filterted texts document does not exist")

    document_ids = list(map(lambda l: int(l.strip().replace(".txt", "")), filtered_documents.open(mode='r')))
    document_ids.sort()

    document_index = pd.Series(document_ids)
    document_index = pd.Series(document_index.index.values, index=document_index.values)

    dimension = len(document_ids)
    similarity_matrix = np.memmap(similarity_matrix_file, dtype='float32', mode='w+', shape=(dimension, dimension))

    load_all_chunks(similarity_matrix, similarity_chunks_folder)
    similarity_matrix.flush()


if __name__ == '__main__':
    try:
        main()
    except (ValueError, FileNotFoundError) as e:
        print(e, file=sys.stderr)
        print("usage: ./compact_similarity_matrix.py <similarity-matrix-chunks> <filtered-texts-file> <compact-similarity-matrix>", file=sys.stderr)