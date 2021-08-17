#!/bin/bash
REPO_ROOT=$(readlink -f $(dirname $(readlink -f $0))/..)
MANIFEST_PATH=$REPO_ROOT/Cargo.toml
LOCAL_SUBSTRATE_REPO_PATH=$1

if [ -z "$LOCAL_SUBSTRATE_REPO_PATH" ]; then
    echo "usage $0 PATH_TO_SUBSTRATE_REPO"
    exit -1
fi

# remove local changes to the file
git -C $REPO_ROOT checkout Cargo.toml > /dev/null
echo '[patch."https://github.com/mangata-finance/substrate"]' >> $MANIFEST_PATH

ALL_DEPENDENCIES=`cargo tree --manifest-path $REPO_ROOT/Cargo.toml --prefix=none | cut -d " " -f 1 | sort | uniq`

# recursively find all packages from local repository and patch them temporarly
for i in `find $LOCAL_SUBSTRATE_REPO_PATH -name Cargo.toml`; do
    MANIFEST_ABS_PATH=$(readlink -f $i)
    git -C $LOCAL_SUBSTRATE_REPO_PATH ls-files --error-unmatch $MANIFEST_ABS_PATH &> /dev/null
    if [ 0 -eq $? ] ; then
        PACKAGE_PATH=`dirname $i`
        PACKAGE_NAME=`sed -n 's/.*name.*=.*\"\(.*\)\"/\1/p' $i | head -1`
        IS_DEPENDENCY=`echo $ALL_DEPENDENCIES| sed 's/ /\n/g' | grep "^$PACKAGE_NAME$"`

        if [ -n "$PACKAGE_NAME" ] && [ -n "$IS_DEPENDENCY" ]; then 
            echo "$PACKAGE_NAME = { path = \"$PACKAGE_PATH\" }" >> $MANIFEST_PATH
        fi
    fi
done
