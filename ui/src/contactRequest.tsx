import React, { useState } from 'react';
import useShrineStore from "./store/shrineStore";
import './styles.css';

const ContactRequestForm = () => {
    const [nodeId, setNodeId] = useState('');
    const sendContactRequest = useShrineStore(state => state.sendContactRequest);
    const { discoverable, setDiscoverable } = useShrineStore(state => ({
        node_id: state.leaderboard.node_id,
        discoverable: state.leaderboard.discoverable,
        setDiscoverable: state.setDiscoverable
    }));

    const handleSubmit = (event: React.FormEvent) => {
        event.preventDefault();
        sendContactRequest(nodeId);
        setNodeId(''); 
    };

    const handleDiscoverableToggle = () => {
        setDiscoverable(!discoverable);
    }; 

    return (
        <div className='contact-card'>
            <div>
                <form className='chat-input' onSubmit={handleSubmit}>
                    <input
                        type="text"
                        value={nodeId}
                        onChange={e => setNodeId(e.target.value)}
                        placeholder="Node ID"
                    />
                    <button className='button' type="submit">Request</button>
                </form>
            </div>
            <button className="button" onClick={handleDiscoverableToggle}>
                {discoverable ? 'you are discoverable' : 'you are hidden'}
            </button>
        </div>
    );
};

export default ContactRequestForm;

