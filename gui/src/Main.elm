module Main exposing (..)

import Browser
import Http
import Maybe
import Html exposing (Html, select, option, button, div, text)
import Html.Attributes exposing (value, selected)
import Html.Events exposing (onClick, onInput)
import Json.Decode as D
import Debug exposing (log)

main = Browser.element { init = init, update = update, subscriptions = subscriptions, view = view }

type alias Model = {
    availablePorts: Remote (List AvailablePort),
    selectedPort: Maybe String }

type Msg
    = GotAvailablePorts (ServerResponse (List AvailablePort))
    | SetSelectedPort String
    | ConnectToSelectedPort
    | RefreshAvailablePorts

-------------------------------------------------------------------------------

type alias AvailablePort = {
    name: String }

getPortList : Cmd Msg
getPortList = httpGet "/list_ports" GotAvailablePorts (D.list (D.map AvailablePort (D.field "name" D.string)))

-------------------------------------------------------------------------------

init : () -> (Model, Cmd Msg)
init _ = (Model Loading Nothing, getPortList)

subscriptions : Model -> Sub Msg
subscriptions model = Sub.none

update : Msg -> Model -> (Model, Cmd Msg)
update msg model =
    case msg of
        GotAvailablePorts result -> 
            let ports = fromResponse result
                selected = case ports of
                                Data ps -> if List.any (\p -> Just p.name == model.selectedPort) ps
                                              then model.selectedPort 
                                              else Maybe.map (\p -> p.name) (List.head ps)
                                _ -> model.selectedPort
            in ({ model | availablePorts = ports, selectedPort = selected}, Cmd.none)
        SetSelectedPort p -> ({ model | selectedPort = Just p}, Cmd.none)
        ConnectToSelectedPort -> (model, Cmd.none)
        RefreshAvailablePorts -> (model, getPortList)

view : Model -> Html Msg
view model = div [] [
    viewAvailablePorts model,
    text (Debug.toString model) ]

viewAvailablePorts model = 
    let portSelector ports = div [] [
            select [ onInput SetSelectedPort ] (List.map (\p -> option [ value p.name, selected (Just p.name == model.selectedPort) ] [text p.name]) ports),
            button [ onClick ConnectToSelectedPort ] [text "Connect"] ]
    in div [] [
        viewRemote model.availablePorts portSelector,
        button [ onClick RefreshAvailablePorts ] [text "Refresh"] ]

-------------------------------------------------------------------------------
-- Helper functions for RPC

type Remote a 
    = Loading
    | ServerError String
    | HttpError Http.Error
    | Data a

type alias ServerResponse a = Result Http.Error (Result String a)

httpGet : String -> (ServerResponse a -> msg) -> D.Decoder a -> Cmd msg
httpGet url constructor decoder =
    let response = D.oneOf [
                        D.field "Err" (D.map Err D.string),
                        D.field "Ok"  (D.map Ok  decoder) ]
    in Http.get { url = url, expect = Http.expectJson constructor response }

fromResponse : ServerResponse a -> Remote a
fromResponse result = case result of
                        Ok (Ok data) -> Data data
                        Ok (Err err) -> ServerError err
                        Err err      -> HttpError err

viewRemote : Remote a -> (a -> Html b) -> Html b
viewRemote remote viewer = 
    case remote of
        Data data       -> viewer data
        Loading         -> text "Loading..."
        ServerError err -> text err
        HttpError err   -> viewHttpError err

viewHttpError : Http.Error -> Html a
viewHttpError err = case err of
                        Http.BadUrl url       -> text ("Invalid URL: " ++ url)
                        Http.Timeout          -> text "Timeout"
                        Http.NetworkError     -> text "Network error"
                        Http.BadStatus status -> text ("Error (status code " ++ String.fromInt status ++ ")")
                        Http.BadBody body     -> text "Could not understand server response"

-------------------------------------------------------------------------------

