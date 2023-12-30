#!/bin/bash
RUSTFLAGS="--cfg=web_sys_unstable_apis" dx serve & npx tailwindcss -i ./src/main.css -o ./public/main.css --watch 
