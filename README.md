# Ratelimit by IP

Goal: I want to be able to rate limit individual routes (e.g. `/login` , `/expensiveOperation`)
easily by client IP address at the proxy layer (close to the edge), and send back a `429 Too Many Requests` response code to clients when that happens.

There are benefits to rate limiting close to the edge- you *could*, and maybe even should,
implement rate limiting at the application layer (both is good). But why waste CPU cycles at the application layer, when you can deny them early on at the edge?

Just like a good ACL rule on edge firewalls dropping packets early on at the edge to avoid pointlessly switching packets the destination is going to drop/deny anyway, with this, you can 
configure your proxy to rate limit by IP any specific route(s) by client IP. Heck, and yes, you could implement billing in this way e.g. 'pay more to get more api requests' but no, bare in mind this does not perform any form of accounting- it will, however, rate limit connections and respond with a http
`429 Too Many Requests` response code to clients when that happens.

## How to use

Essentially two steps:

1. Compile & build this ratelimit program
2. Configure apache to use it

### Compile / Build ratelimitbyip

```shell
git checkout https://github.com/chrisjsimpson/ratelimitbyip.git
cargo build
```

> [!IMPORTANT]
> You **must** `chown` (change the ownership of your `ratelimit` binary) to the same user apache is
running at (e.g. user `www-data` by default on ubuntu) so that apache can exec the `ratelimit` binary when apache starts.

### Configure apache to use `ratelimitbyip`

Example Apache configuration:

`/etc/apache2/sites-available/example.com.conf`

```shell
<VirtualHost *:80>
        ServerName example.com
        LogLevel trace6 rewrite:trace6

        RewriteEngine On
        # I'd like to ratelimit the 'protect.html' page at
        # http://example.com/protect.html' using the rate limiter
        RewriteMap ratelimitbyip "prg:/path/to/ratebyip" www-data:www-data

        ErrorDocument 429 /var/www/html/ratelimited.html
        RewriteCond "%{REQUEST_URI}" =/protect.html
        RewriteRule ".*" "${ratelimitbyip:%{REMOTE_ADDR}|%{REQUEST_URI}}"
        RewriteRule /ratelimited / [R=429]

        # Now I'd also like to share the ratelimit, limiting both
        # protect.html (already done with above config), but also
        # http://example.com/login.html
        RewriteCond "%{REQUEST_URI}" =/login.html
        RewriteRule ".*" "${ratelimitbyip:%{REMOTE_ADDR}|%{REQUEST_URI}}"

        # Now I'd like to add an exclusive (not shared) ratelimit
        # to the address http://example.com/ (homepage) so that it's
        # rate limit is not shared between the other rate limited
        # paths. In other words, for a given client_ip, the 
        # earlier rate-limited paths
        # (/protect.html and /login.html) will get half the time
        # compared the / path.
        RewriteMap ratelimitbyiphomepage "prg:/path/to/ratebyip" www-data:www-data
        RewriteCond "%{REQUEST_URI}" =/
        RewriteRule ".*" "${ratelimitbyiphomepage:%{REMOTE_ADDR}|%{REQUEST_URI}}"

        ServerAdmin webmaster@localhost
        DocumentRoot /var/www/html

        ErrorLog ${APACHE_LOG_DIR}/example.com.error.log
        CustomLog ${APACHE_LOG_DIR}/example.com.access.log combined
</VirtualHost>

```

## Background

I was surprised how complex/involved is was to rate limit by client IP address using Envoy Proxy at layer 7. In fact, I'm yet to be able to configure that. With envoy you can rate limit at the network layer (https://github.com/envoyproxy/envoy/issues/19685) ([layer 3](https://en.wikipedia.org/wiki/Network_layer)) but not (afaik) at the application layer (l7) in `http` land.

Many rabbit holes later, and a chance to play with rust. Of course, we land on Apache and an interesting approach. Enter this repo: Apache rate limit by IP

### Related

- [What's the best method to implement rate limiting with Apache?](https://serverfault.com/questions/1160808/whats-the-best-method-to-implement-rate-limiting-with-apache)
- [Envoy - Rate limit (proto)](https://www.envoyproxy.io/docs/envoy/latest/api-v3/extensions/filters/http/ratelimit/v3/rate_limit.proto)
- Thanks to [pelikan-io/rustcommon/ratelimit](https://github.com/pelikan-io/rustcommon/tree/main/ratelimit)