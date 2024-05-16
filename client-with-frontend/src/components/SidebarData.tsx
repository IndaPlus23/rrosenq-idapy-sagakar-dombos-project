import ChatIcon from '@mui/icons-material/Chat';
import BubbleChartIcon from '@mui/icons-material/BubbleChart';
import SettingsIcon from '@mui/icons-material/Settings';
import PermIdentityIcon from '@mui/icons-material/PermIdentity';


export const SidebarData = [

    {
        title: "Chat",
        icon: <ChatIcon />,
        link: "/chat"
    },

    {
        title: "Direct message",
        icon: <BubbleChartIcon />,
        link: "/dm"
    },


    {
        title: "Profile",
        icon: <PermIdentityIcon />,
        link: "/profile"
    },

    {
        title: "Settings",
        icon: <SettingsIcon />,
        link: "/settings"
    },

]