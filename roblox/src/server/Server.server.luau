--!native
--!optimize 2

local HttpService = game:GetService("HttpService")
local ReplicatedStorage = game:GetService("ReplicatedStorage")

-- Если сервер на Replit, используйте его публичный URL
-- Например: local URL = "https://262001b6-3c84-4c27-93ce-da3b51264469-00-1ena7c2vnwci4.sisko.replit.dev/"
-- Для локального тестирования: local URL = "http://localhost:3000/"
local URL = "http://localhost:3000/"

ReplicatedStorage.Request.OnServerInvoke = function(_player, data: buffer)
        return buffer.fromstring(HttpService:PostAsync(URL, buffer.tostring(data)))
end
