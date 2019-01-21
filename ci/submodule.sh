if [ "$TRAVIS_OS_NAME" == "linux" ]; then
  sed -i 's/git@github.com:/https:\/\/github.com\//' .gitmodules
else
  sed -i '' 's/git@github.com:/https:\/\/github.com\//' .gitmodules
fi

git submodule update --init --recursive
