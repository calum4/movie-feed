
# Still under active development!

<h1 align="center">
  <br>
  Movie Feed
  <br>
</h1>

<h4 align="center"></h4>

<p align="center">
</p>

<p align="center">
  <a href="#changelog">Changelog</a> •
  <a href="#usage">Usage</a> •
  <a href="#license">License</a> •
  <a href="#contributing">Contributing</a>
</p>

## Changelog

The full changelog can be found at [CHANGELOG.md](CHANGELOG.md)

## Motivation

I found myself wanting a better way to keep tabs on upcoming tv and movies from my favourite actors/crew. After 
considering a few different options, I settled on creating an API which queried [TMDB](https://www.themoviedb.org/), 
and then formatted the resultant data into an RSS feed.

I then consume this feed with the feed reader of my choice, [FreshRSS](https://github.com/FreshRSS/FreshRSS) (other feed
readers are available).

## Usage

**Note** - Under no circumstance do I recommend exposing Movie Feed to the internet.

Currently, client authentication is not supported and as such could be misused if exposed to an untrusted network. If
you wish to apply authentication, you could use HTTP basic auth provided by a reverse proxy such as Nginx.

## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).
