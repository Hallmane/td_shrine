import React, { useState, useEffect } from 'react';
import useShrineStore from './store/shrineStore';

const Chat: React.FC = () => {
    const chatHistory = useShrineStore(state => state.chat.chat_history);
    const sendChatMessage = useShrineStore(state => state.sendChatMessage);
    const updateLeaderboard = useShrineStore(state => state.updateLeaderboard);
    const [message, setMessage] = useState("");

    useEffect(() => {
        console.log('Chat history:', chatHistory);
    }, [chatHistory]);

    const handleSendMessage = async (e: React.FormEvent) => {
        e.preventDefault();
        if (message.trim() !== '') {
            await sendChatMessage(message);
            await updateLeaderboard();
            setMessage('');
        }
    };

    return (
        <div className="chat-container">
            <div className="chat-history">
                {chatHistory && chatHistory.length > 0 ? (
                    chatHistory.map((msg, index) => (
                        <div key={index} className="chat-message">
                            <strong>{msg.sender}:</strong> {msg.content}
                        </div>
                    ))
                ) : (
                    <div className="no-messages">No messages yet</div>
                )}
            </div>
            <form className='chat-input' onSubmit={handleSendMessage}>
                <input 
                    type="text" 
                    value={message} 
                    onChange={e => setMessage(e.target.value)} 
                    placeholder="wwtdd..."
                />
                <button type="submit">Send</button>
            </form>
        </div>
    );
};

export default Chat;


//import React, { useState, useEffect } from 'react';
//import useShrineStore from './store/shrineStore';
//
//const Chat: React.FC = () => {
//    const chatHistory = useShrineStore(state => state.chat.chat_history);
//    const sendChatMessage = useShrineStore(state => state.sendChatMessage);
//    const updateLeaderboard = useShrineStore(state => state.updateLeaderboard);
//    const [message, setMessage] = useState("");
//
//    useEffect(() => {
//        console.log('Chat history:', chatHistory);
//    }, [chatHistory]);
//
//    const handleSendMessage = async (e: React.FormEvent) => {
//        e.preventDefault();
//        if (message.trim() !== '') {
//            await sendChatMessage(message);
//            await updateLeaderboard();
//            setMessage('');
//        }
//    };
//
//    return (
//        <div className="chat-container">
//            <div className="chat-history">
//                {chatHistory && chatHistory.length > 0 ? (
//                    chatHistory.map((msg, index) => (
//                        <div key={index} className="chat-message">
//                            <strong>{msg.sender}:</strong> {msg.content}
//                        </div>
//                    ))
//                ) : (
//                    <div className="no-messages">No messages yet</div>
//                )}
//            </div>
//            <form onSubmit={handleSendMessage}>
//                <input 
//                    type="text" 
//                    value={message} 
//                    onChange={e => setMessage(e.target.value)} 
//                    placeholder="wwtdd..."
//                />
//                <button type="submit">Send</button>
//            </form>
//        </div>
//    );
//};
//
//export default Chat;

/////////////////////////////// sss ///////////////

//import React, { useState, useEffect } from 'react';
//import useShrineStore from './store/shrineStore';
//
//const Chat: React.FC = () => {
//    const chatHistory = useShrineStore(state => state.chat.chat_history);
//    const sendChatMessage = useShrineStore(state => state.sendChatMessage);
//    const updateChat = useShrineStore(state => state.updateChat);
//    const [message, setMessage] = useState("");
//
//    useEffect(() => {
//        console.log('Chat history:', chatHistory);
//    }, [chatHistory]);
//
//    const handleSendMessage = (e: React.FormEvent) => {
//        e.preventDefault();
//        if (message.trim() !== '') {
//            sendChatMessage(message);
//            updateChat();
//            setMessage('');
//        }
//    };
//
//    return (
//        <div className="chat-container">
//            <div className="chat-history">
//                {chatHistory && chatHistory.length > 0 ? (
//                    chatHistory.map((msg, index) => (
//                        <div key={index} className="chat-message">
//                            <strong>{msg.sender}:</strong> {msg.content}
//                        </div>
//                    ))
//                ) : (
//                    <div className="no-messages">No messages yet</div>
//                )}
//            </div>
//            <form onSubmit={handleSendMessage}>
//                <input 
//                    type="text" 
//                    value={message} 
//                    onChange={e => setMessage(e.target.value)} 
//                    placeholder="wwtdd..."
//                />
//                <button type="submit">Send</button>
//            </form>
//        </div>
//    );
//};
//
//export default Chat;




//import React, { useState, useEffect } from 'react';
//import useShrineStore from './store/shrineStore';
//
//const Chat: React.FC = () => {
//    const chatHistory = useShrineStore(state => state.chat.chat_history);
//    const sendChatMessage = useShrineStore(state => state.sendChatMessage);
//    const [message, setMessage] = useState("");
//
//    useEffect(() => {
//        console.log('Chat history:', chatHistory);
//    }, [chatHistory]);
//
//    const handleSendMessage = (e: React.FormEvent) => {
//        e.preventDefault();
//        if (message.trim() !== '') {
//            sendChatMessage(message);
//            setMessage('');
//        }
//    };
//
//    return (
//        <div className="chat-container">
//            <div className="chat-history">
//                {chatHistory.map((msg, index) => (
//                    <div key={index} className="chat-message">
//                        <strong>{msg.sender}:</strong> {msg.content}
//                    </div>
//                ))}
//            </div>
//            <form onSubmit={handleSendMessage}>
//                <input 
//                    type="text" 
//                    value={message} 
//                    onChange={e => setMessage(e.target.value)} 
//                    placeholder="wwtdd..."
//                />
//                <button type="submit">Send</button>
//            </form>
//        </div>
//    );
//};
//
//export default Chat;
//