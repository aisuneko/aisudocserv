# aisudocserv
Small file server/search engine for local HTML docs.

Powered by warp+tantivy+tera. (~~didn't expect it to be this complex...~~)

## Usage
Download the binary from [Releases](https://github.com/aisuneko/aisudocserv/releases) and put it directly **under the root** of your documentation/site folder. Run it and go to http://localhost:3030/search for your queries. Everything else is served under /, allowing you to browse it just like a normal online site.
(Only x86_64 for now, sorry...)

## Note
This is a crude, half-baked utility; more work is needed to make it more production-ready (For example, I temporarily gave up with the CSS part so the frontend is kinda terrible).