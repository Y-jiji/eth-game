cd ./.local
if command -v axel &> /dev/null
then
    echo "use axel"
    axel -n 8 https://download.pytorch.org/libtorch/cu117/libtorch-cxx11-abi-shared-with-deps-2.0.1%2Bcu117.zip
else
    echo "use wget"
    wget https://download.pytorch.org/libtorch/cu117/libtorch-cxx11-abi-shared-with-deps-2.0.1%2Bcu117.zip
fi
unzip libtorch-shared-with-deps-latest-*.zip.*
cd ..
