import React, { useState, useEffect } from 'react';
import useShrineStore from './store/shrineStore';
import useChatStore from './store/chatStore';

const Chat: React.FC = () => {
    const chatHistory = useChatStore(state => state.chat.chat_history);
    const sendChatMessage = useChatStore(state => state.sendChatMessage);
    const [message, setMessage] = useState("");

    useEffect(() => {
        console.log('Chat history:', chatHistory);
    }, [chatHistory]);

    const handleSendMessage = async (e: React.FormEvent) => {
        e.preventDefault();
        if (message.trim() !== '') {
            await sendChatMessage(message);
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