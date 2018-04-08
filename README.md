# License Bot

License bot adds a comment to Reddit submissions that link to any Github repository which doesn't contain a license file.

## Setup

Create a .env file in the root project directory, based off of the .env.example file. 

```
// Required on OSX >=10.11

brew install openssl

export OPENSSL_INCLUDE_DIR="$(brew --prefix openssl)/include"
export OPENSSL_LIB_DIR="$(brew --prefix openssl)/lib"

cargo build
```

## Run

##### Local

```
cargo run
```

##### Heroku

1. Create a new Heroku app
1. Install the `Heroku Scheduler` add-on
1. Add Heroku as a git remote `heroku git:remote -a license-bot`
1. Tell Heroku to use the Rust buildpack `heroku buildpacks:set https://github.com/emk/heroku-buildpack-rust.git`
1. Push to the Heroku repo `git push heroku master`
1. Setup the heroku scheduler to run `./target/release/license-bot`

* Resouces
    * https://devcenter.heroku.com/articles/scheduler
    * https://github.com/emk/heroku-buildpack-rust
    
## Dependencies

This project is dependant on a forked version of RAWR (Rust API Wrapper for Reddit) in order to allow posting a Reddit comment at the root of a submission after iterating through all of its comments.
