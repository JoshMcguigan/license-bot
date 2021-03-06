# License Bot

[License bot](https://www.reddit.com/user/license-bot) adds a comment to Reddit submissions that link to any Github repository which doesn't contain a license file.

## Setup

Create a `.env` file in the root project directory, based off of the `.env.example` file. 

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
// First be sure to complete the setup above

cargo run 
```

##### Heroku

1. Create a new Heroku app
1. Install the `Heroku Scheduler` add-on
1. Add Heroku as a git remote `heroku git:remote -a license-bot`
1. Tell Heroku to use the Rust buildpack `heroku buildpacks:set https://github.com/emk/heroku-buildpack-rust.git`
1. Push to the Heroku repo `git push heroku master`
1. Setup the environment variables on Heroku
1. Setup the heroku scheduler to run `./target/release/license-bot`
1. Test run the bot with `heroku run ./target/release/license-bot`

* Resouces
    * https://devcenter.heroku.com/articles/scheduler
    * https://github.com/emk/heroku-buildpack-rust
    
    
## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
