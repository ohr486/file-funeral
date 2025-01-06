require 'thin'
require_relative './app'

Faye::WebSocket.load_adapter('thin')

run App
