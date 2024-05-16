import Chatbox from '../components/Chatbox';
import MessageDisplay from '../components/ChatDisplay';
import ChannelMenu from '../components/ChannelMenu';
import { useState, useEffect } from 'react';
import Message from '../components/Message';
import { invoke } from '@tauri-apps/api';
import { Event } from '@tauri-apps/api/event';
import { listen } from '@tauri-apps/api/event';

interface DM {
    id: string;
    name: string;
    messages: Message[];
}

interface DMPageProps{
    messageDisplayRef: React.RefObject<HTMLDivElement>;
    userName: string;
}

function DMPage({ messageDisplayRef, userName }: DMPageProps) {

    const [activeDM, setActiveDM] = useState<DM | null>(null);
    const [DMs, setDMs] = useState<DM[]>([]);
    const [updated, setupdated] = useState<boolean>(false);
    const [firstDM, setFirstDM] = useState<boolean>(true)

    const switchDM = (DMId: string) => {
        const DM = DMs.find(c => c.id === DMId);
        if (DM) {
            setActiveDM(DM);
        }
    };

    const createDM = (channelId: string) => {
        const newDM: DM = {
            id: channelId,
            name: channelId,
            messages: [],
        };
        DMs.push(newDM);
        if (firstDM) {
            setActiveDM(newDM);
            setFirstDM(false)
        }
    }

    useEffect(() => {
        async function listen_messages() {
            await listen('recieve_message', (event: Event<Message>) => {
                const DM = DMs.find(d => d.id === event.payload.channel);
                if (DM) {
                    DM.messages.push(event.payload)
                    setupdated(true)
                }
            });
        }
        listen_messages()
    }, [])

    const sendMessage = (message: string) => {
        invoke('send_message', { message: message, target: activeDM ? activeDM.id : ""})
    };

    useEffect(() => {
        if (messageDisplayRef.current) {
          messageDisplayRef.current.scrollTop = messageDisplayRef.current.scrollHeight;
        }
        setupdated(false)
      }, [updated])
    
    return (
        <div className='Chat'>
            <div className='DMMenu'>
                <ChannelMenu
                    header={"Direct Messages"}
                    onChannelSelect={switchDM}
                    activeChannelId={activeDM ? activeDM.id : ''}
                    isDM={true}
                    onChannelCreate={createDM}
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
