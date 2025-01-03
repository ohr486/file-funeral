require "webview_ruby"

webview = WebviewRuby::Webview.new
webview.set_title("Ruby DEMO")
webview.set_size(960, 680)
webview.navigate("https://en.m.wikipedia.org/wiki/Main_Page")
webview.run
