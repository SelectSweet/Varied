# Varied

Required command line tools

- FFmpeg: 
  Install from you package manager or download from [here](https://ffmpeg.org/download.html) ensuring that it's available on your path

- RClone
  Download from the [RClone website](https://rclone.org/install/), regardless of the way you install it ensure that it is available via your path, then run the [config setup](https://rclone.org/docs/) that matches your object storage

- sea-orm-cli: 
  Install Using Cargo

  `cargo install sea-orm-cli`


## Install

### Source

`git clone https://github.com/SelectSweet/Varied.git `


## Config

in the varied.toml file follow the comments given for config instructions


## Run

`sea-orm-cli migrate up`

`cargo run --release`

