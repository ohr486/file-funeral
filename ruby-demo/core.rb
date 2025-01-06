require "webview_ruby"

webview = WebviewRuby::Webview.new(debug: true)
webview.set_title("Ruby DEMO")
webview.set_size(960, 680)
#webview.navigate("https://en.m.wikipedia.org/wiki/Main_Page")
#webview.navigate("file:///Volumes/dock-SSD/Git/github.com/ohr486/file-funeral/ruby-demo/index.html")
webview.navigate("https://localhost:9292")
webview.run
