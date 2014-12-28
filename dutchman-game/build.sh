cargo build
# cp target/dutchman_game*.dll ./dutchman_game.dll
rm ./dutchman_game*.dll
cp target/dutchman_game*.dll ./dutchman_game-${RANDOM}.dll
