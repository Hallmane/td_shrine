import React, { useState, useEffect } from "react";
import useShrineStore from "./store/shrineStore";
import { Leaderboard } from "./leaderboard";
import ContactRequestForm from "./contactRequest";
import IncomingRequests from "./incomingRequests";
import OutgoingRequests from "./outgoingRequests";
import Chat from "./Chat";
import './styles.css';

const BASE_URL = import.meta.env.BASE_URL;

if (window.our) window.our.process = BASE_URL?.replace("/", "");

const App: React.FC = () => {
   const node_id = useShrineStore(state => state.leaderboard.node_id);
   const incoming_contact_requests = useShrineStore(state => state.leaderboard.incoming_contact_requests);
   const pending_contact_requests = useShrineStore(state => state.leaderboard.pending_contact_requests);
   const [audioPlaying, setAudioPlaying] = useState(false);
   const audioRef = React.useRef<HTMLAudioElement>(null);

   useEffect(() => {
      console.log('App component mounted');
      console.log('node_id:', node_id);
   }, [node_id]);

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
            <button className="button" onClick={() => useShrineStore.getState().addRespect(node_id)}>✞ Pay Respects ✞</button>
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
