"""Module to deal with text acquisition."""

import gzip
import os

import requests

def _etextno_to_uri_subdirectory(etextno):
    """Returns the subdirectory that an etextno will be found in a gutenberg
    mirror. Generally, one finds the subdirectory by separating out each digit
    of the etext number, and uses it for a directory. The exception here is for
    etext numbers less than 10, which are prepended with a 0 for the directory
    traversal.

    >>> _etextno_to_uri_subdirectory(1)
    '0/1'
    >>> _etextno_to_uri_subdirectory(19)
    '1/19'
    >>> _etextno_to_uri_subdirectory(15453)
    '1/5/4/5/15453'
    """
    str_etextno = str(etextno).zfill(2)
    all_but_last_digit = list(str_etextno[:-1])
    subdir_part = "/".join(all_but_last_digit)
    subdir = "{0}/{1}".format(subdir_part, etextno)  # etextno not zfilled
    return subdir

def _check_mirror_exists(mirror):
    response = requests.head(mirror)
    if not response.ok:
        raise Exception(
            'Could not reach Gutenberg mirror "{0:s}".'
            .format(mirror))


def _format_download_uri(etextno, mirror):
    """Returns the download location on the Project Gutenberg servers for a
    given text.

    Raises:
        UnknownDownloadUri: If no download location can be found for the text.
    """
    uri_root = mirror.strip().rstrip('/')
    _check_mirror_exists(uri_root)

    extensions = ('.txt', '-8.txt', '-0.txt')
    for extension in extensions:
        path = _etextno_to_uri_subdirectory(etextno)
        uri = '{root}/{path}/{etextno}{extension}'.format(
            root=uri_root,
            path=path,
            etextno=etextno,
            extension=extension)
        response = requests.head(uri)
        if response.ok:
            return uri

    raise Exception('Failed to find {0} on {1}.'
                                      .format(etextno, uri_root))


def load_etext(etextno, refresh_cache=False, mirror=None):
    """Returns a unicode representation of the full body of a Project Gutenberg
    text. After making an initial remote call to Project Gutenberg's servers,
    the text is persisted locally.

    """
    if not isinstance(etextno, int) or etextno <= 0:
        raise Exception("Invalid text number.")

    download_uri = _format_download_uri(etextno, mirror)
    response = requests.get(download_uri)
    text = response.text

    return text
