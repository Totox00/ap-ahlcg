wasm-pack build --target web --out-dir ./www/pkg
rm -rf ./www/.git
cp ./Id.py ../ahlcg/Id.py
cp ./Data.py ../ahlcg/Data.py
