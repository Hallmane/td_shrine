import useShrineStore from './store/shrineStore';
import './styles.css';

const OutgoingRequests = () => {
    const { pending_contact_requests} = useShrineStore(state => ({
        incoming_contact_requests: state.leaderboard.incoming_contact_requests,
        pending_contact_requests: state.leaderboard.pending_contact_requests,
        acceptContact: state.acceptContactRequest,
    }));

    return (
        <div className='contact-card'>
            <h3>Outgoing Contact Requests</h3>
                <ul>
                    {pending_contact_requests.map((nodeId, index) => (
                        <div key={index} className="leaderboard-entry">
                        <span>{nodeId}</span>
                    </div>
                    ))}
                </ul>
            </div>
    );
};

export default OutgoingRequests;