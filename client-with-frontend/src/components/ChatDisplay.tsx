interface MessageDisplayProps {
  messages: string[];
}

function MessageDisplay({ messages }: MessageDisplayProps) {
return (
  <div className="Box">
    {messages.map((message, index) => (
      <div key={index} className="Message">
          <p>{message}</p>
      </div>
    ))}
  </div>
);
}

export default MessageDisplay;
