rd udata /s/q
md udata
pushd udata

wget https://www.iana.org/assignments/language-subtag-registry/language-subtag-registry

wget ftp://unicode.org/Public/15.0.0/ucd/Blocks.txt
wget ftp://unicode.org/Public/15.0.0/ucd/NamesList.txt
wget ftp://unicode.org/Public/15.0.0/ucd/UnicodeData.txt

popd
