import useShrineStore from './store/shrineStore';
import './styles.css';

const IncomingRequests = () => {
    const { incoming_contact_requests, acceptContact} = useShrineStore(state => ({
        incoming_contact_requests: state.leaderboard.incoming_contact_requests,
        pending_contact_requests: state.leaderboard.pending_contact_requests,
        acceptContact: state.acceptContactRequest,
        //declineContact: state.declineContactRequest,
    }));

    return (
        <div className='contact-card'>
            <div>Incoming Contact Requests</div>
            {incoming_contact_requests.map((nodeId, index) => (
                <div key={index}>
                    {nodeId}
                    <button className="button" onClick={() => acceptContact(nodeId)}>Accept</button>
                </div>
            ))}
        </div>
    );
};

export default IncomingRequests;
