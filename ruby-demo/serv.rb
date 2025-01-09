require 'webrick'
require 'json'

# サーバーの設定
server = WEBrick::HTTPServer.new(Port: 8000)

# サーブレットを作成
#class MyServlet < WEBrick::HTTPServlet::AbstractServlet
#  def do_POST(request, response)
#    response.status = 200
#    response['Content-Type'] = 'application/json'
#
#    post_data = JSON.parse(request.body)
#
#    # ここで受け取ったデータを処理する
#    response.body = post_data.to_json
#  end
#end

# 新しいサーブレットを作成
class AnotherServlet < WEBrick::HTTPServlet::AbstractServlet
  def do_GET(request, response)
    response.status = 200
    response['Content-Type'] = 'text/plain'

    response.body = 'Hello from another endpoint!'
  end
end

# 既存のエンドポイントにサーブレットをマウント
#server.mount('.', MyServlet)

# 新しいエンドポイントにサーブレットをマウント
server.mount('/', AnotherServlet)

# Ctrl+Cでサーバーを停止するためのシグナルハンドラを設定
trap('INT') { server.shutdown }

# サーバーを起動
server.start


