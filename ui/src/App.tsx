import React, { useState, useEffect } from "react";
import KinodeClientApi from "@kinode/client-api";
import useShrineStore from "./store/shrineStore";
import { Leaderboard } from "./leaderboard";
import ContactRequestForm from "./contactRequest";
import IncomingRequests from "./incomingRequests";
import OutgoingRequests from "./outgoingRequests";
import Chat from "./Chat";
import './styles.css';

const BASE_URL = import.meta.env.BASE_URL;
const PROXY_TARGET = `${(import.meta.env.VITE_NODE_URL || "http://localhost:8080")}${BASE_URL}`;
const WEBSOCKET_URL = import.meta.env.DEV ? `${PROXY_TARGET.replace('http', 'ws')}` : undefined; //why dev?

if (window.our) window.our.process = BASE_URL?.replace("/", "");

const App: React.FC = () => {
   const setApi = useShrineStore();
   const node_id = useShrineStore(state => state.leaderboard.node_id); //remove this from backend as well
   const incoming_contact_requests = useShrineStore(state => state.leaderboard.incoming_contact_requests);
   const pending_contact_requests = useShrineStore(state => state.leaderboard.pending_contact_requests);
   const [connected, setConnected] = useState(false);
   const [audioPlaying, setAudioPlaying] = useState(false);
   const audioRef = React.useRef<HTMLAudioElement>(null);

   useEffect(() => {
      const kinnect = () => {
         const newApi = new KinodeClientApi({
            uri: WEBSOCKET_URL,
            nodeId: window.our.node,
            processId: window.our.process,
            //onClose: () => {},
            onOpen: (_event, _api) => {console.log("Connected to kinode")},
            //onMessage: (json, _api) => {handleWsMessage(json);},
            onError: (e) => {console.log("websocket error: ", e)},
         });

         //setApi(newApi);

      };
   })

   useEffect(() => {
      console.log('App component mounted');
      console.log('node_id:', window.our.node);
   }, []);

   const toggleAudio = () => {
      if (audioRef.current) {
         if (audioPlaying) {
            audioRef.current.pause();
            setAudioPlaying(false);
         } else {
            audioRef.current.play();
            setAudioPlaying(true);
         }
      }
   };

   return (
      <div className="outer-container">
         <div>
            <Chat />
         </div>
         <div className="middle-column">
            <strong id="play-audio" onClick={toggleAudio} className="button audio-button">
               {audioPlaying ? '</3' : '<3'}
            </strong>
            <h1>Terry A. Davis</h1>
            <div className="terry-background">
               <img src="./assets/terry.gif" alt="the hero no one deserved" className="terry-image" />
            </div>
            <h2>1969 - 2018</h2>
            <h4>Lost but never forgotten</h4>
            <button className="button" onClick={() => useShrineStore.getState().addRespect(window.our.node)}>✞ Pay Respects ✞</button>
            <audio id="audio-player" ref={audioRef} src="./assets/wowy.mp3" loop></audio>
         </div>

         <div className="right-column">
            <div className="top-section">
               <Leaderboard />
            </div>
            <div className="bottom-section">
               {(incoming_contact_requests.length > 0) && (
                  <IncomingRequests />
               )}
               {(pending_contact_requests.length > 0) && (
                  <OutgoingRequests />
               )}
               <ContactRequestForm />
            </div>
         </div>
      </div>
   );
};

export default App;
