import React, { useEffect } from 'react';
import useShrineStore from './store/shrineStore';
import './styles.css'; 

export const Leaderboard = () => {

    const { node_id, leaderboard, updateLeaderboard} = useShrineStore(state => ({
        node_id: state.leaderboard.node_id,
        leaderboard: state,
        discoverable: state.leaderboard.discoverable,
        updateLeaderboard: state.updateLeaderboard,
    }));

    useEffect(() => {
        updateLeaderboard();
    }, [updateLeaderboard]);


    return (
        <div className="card">
            <h2>Respectors</h2>
            <li className='leaderboard-entry'>
                <span > Node </span> <span > Respects </span>
            </li>
            <ul>
                {Object.entries(leaderboard.leaderboard.stats).sort((a, b) => b[1].respects - a[1].respects).map(([nodeId, entry]) => (
                    <li className='leaderboard-entry' key={nodeId}>
                        <span>{nodeId} </span> <span>{entry.respects} </span>
                    </li>
                ))}
            </ul>
        </div>
    );
};