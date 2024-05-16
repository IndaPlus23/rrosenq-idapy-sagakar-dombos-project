import Chatbox from '../components/Chatbox';
import MessageDisplay from '../components/ChatDisplay';
import ChannelMenu from '../components/ChannelMenu';
import { useState, useEffect } from 'react';

interface DM {
    id: string;
    name: string;
    messages: string[];
}

interface DMPageProps{
    messageDisplayRef: React.RefObject<HTMLDivElement>;
    userName: string;
}

function DMPage({ messageDisplayRef, userName }: DMPageProps) {

    const [activeDM, setActiveDM] = useState<DM | null>(null);
    const [DMs, setDMs] = useState<DM[]>([]);

    const switchDM = (DMId: string) => {
        const DM = DMs.find(c => c.id === DMId);
        if (DM) {
            setActiveDM(DM);
        } else {
            const newDM: DM = {
                id: DMId,
                name: DMId,
                messages: [],
            };
            setActiveDM(newDM);
            setDMs(prevDMs => [...prevDMs, newDM]);
        }
    };

    const sendMessage = (message: string) => {
        if (activeDM) {
            const updatedDM = { ...activeDM, messages: [...activeDM.messages, message] };
            setActiveDM(updatedDM);
            setDMs(prevDMs => prevDMs.map(c => c.id === activeDM.id ? updatedDM : c));
        }
    };

    useEffect(() => {
        if (messageDisplayRef.current) {
          messageDisplayRef.current.scrollTop = messageDisplayRef.current.scrollHeight;
        }
      }, [activeDM ? activeDM.messages : ''])
    
    return (
        <div className='Chat'>
            <div className='DMMenu'>
                <ChannelMenu
                    header={"Direct Messages"}
                    onChannelSelect={switchDM}
                    activeChannelId={activeDM ? activeDM.id : ''}
                    />
            </div>
            <div ref={messageDisplayRef} className='ChatDisplay'>
                <MessageDisplay messages={activeDM ? activeDM.messages : []} />
            </div>
            <div className='Chatbox'>
                <Chatbox sendMessage={sendMessage} userName={userName}/>
            </div>
        </div>
    );
}

export default DMPage;
