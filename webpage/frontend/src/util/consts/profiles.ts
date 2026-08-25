import profilePlaceholder from '../../assets/profile_placeholder.jpg';

export interface Teammate {
	name: string,
	role: string,
	study: string,
	img_src: string
};

export const team_info: Teammate[] = [
	{
		name: "Vikas Gowda",
		role: "Professor Lead",
		study: "",
		img_src: profilePlaceholder,
	},
	{
		name: "Connor Hill",
		role: "Programmer",
		study: "Computer Science",
		img_src: profilePlaceholder,
	},
	{
		name: "Andy Zheng",
		role: "Programmer",
		study: "Computer Science",
		img_src: profilePlaceholder,
		
	},
	{
		name: "Deris O'Malley",
		role: "Programmer",
		study: "Computer Science",
		img_src: profilePlaceholder,
	},
	{
		name: "Genensis BautistaSanchez",
		role: "Editor",
		study: "Computer Science",
		img_src: profilePlaceholder,
	},
];

export interface Socials {
    github_link: string,
    linkedin_link: string,
}

export interface Profile {
    teammate_profile: Teammate,
    background_description: string,
    social_links: Socials
}

export interface ProfileBios {
    [key: string]: Profile
}

export const profile_bios: ProfileBios = {
    "genensisbautistasanchez": {
		teammate_profile: {
            name: "Genensis BautistaSanchez",
            role: "Editor",
            study: "Computer Science and Innovation",
            img_src: profilePlaceholder
        },
        background_description: `Lorem ipsum dolor sit amet, consectetur adipiscing elit. 
                                Nullam vulputate orci in mattis accumsan. Etiam fermentum hendrerit porta. Nam malesuada mauris enim, sed rhoncus nibh aliquet eu. Nunc vestibulum tempus mattis. Etiam consectetur turpis ultricies lectus dictum feugiat. Proin euismod id nisi tempor placerat. Phasellus eu faucibus massa, ut interdum orci. Duis sed tincidunt eros. Integer accumsan egestas facilisis. Ut porta mollis nisi, vel tincidunt nisi imperdiet vel. In imperdiet sodales nulla, non venenatis arcu pulvinar id. Quisque et metus quis libero facilisis iaculis et ullamcorper magna. Nam risus risus, blandit non consectetur quis, lobortis eget erat. In hac habitasse platea dictumst. Mauris vel erat suscipit, iaculis nibh vitae, tempus tortor. Quisque orci elit, pharetra et felis sit amet, suscipit consectetur diam.`,
        social_links: {
            github_link: "test",
            linkedin_link: "test"
        }
	},
    "vikasgowda": {
		teammate_profile: {
            name: "Vikas Gowda",
            role: "Research Lead",
            study: "",
            img_src: profilePlaceholder
        },
        background_description: `Lorem ipsum dolor sit amet, consectetur adipiscing elit. 
                                Nullam vulputate orci in mattis accumsan. Etiam fermentum hendrerit porta. Nam malesuada mauris enim, sed rhoncus nibh aliquet eu. Nunc vestibulum tempus mattis. Etiam consectetur turpis ultricies lectus dictum feugiat. Proin euismod id nisi tempor placerat. Phasellus eu faucibus massa, ut interdum orci. Duis sed tincidunt eros. Integer accumsan egestas facilisis. Ut porta mollis nisi, vel tincidunt nisi imperdiet vel. In imperdiet sodales nulla, non venenatis arcu pulvinar id. Quisque et metus quis libero facilisis iaculis et ullamcorper magna. Nam risus risus, blandit non consectetur quis, lobortis eget erat. In hac habitasse platea dictumst. Mauris vel erat suscipit, iaculis nibh vitae, tempus tortor. Quisque orci elit, pharetra et felis sit amet, suscipit consectetur diam.`,
        social_links: {
            github_link: "test",
            linkedin_link: "test"
        }
	},
	"connorhill": {
		teammate_profile: {
            name: "Connor Hill",
            role: "Research Programmer",
            study: "Computer Science and Innovation",
            img_src: profilePlaceholder
        },
        background_description: `Lorem ipsum dolor sit amet, consectetur adipiscing elit. 
                                Nullam vulputate orci in mattis accumsan. Etiam fermentum hendrerit porta. Nam malesuada mauris enim, sed rhoncus nibh aliquet eu. Nunc vestibulum tempus mattis. Etiam consectetur turpis ultricies lectus dictum feugiat. Proin euismod id nisi tempor placerat. Phasellus eu faucibus massa, ut interdum orci. Duis sed tincidunt eros. Integer accumsan egestas facilisis. Ut porta mollis nisi, vel tincidunt nisi imperdiet vel. In imperdiet sodales nulla, non venenatis arcu pulvinar id. Quisque et metus quis libero facilisis iaculis et ullamcorper magna. Nam risus risus, blandit non consectetur quis, lobortis eget erat. In hac habitasse platea dictumst. Mauris vel erat suscipit, iaculis nibh vitae, tempus tortor. Quisque orci elit, pharetra et felis sit amet, suscipit consectetur diam.`,
        social_links: {
            github_link: "test",
            linkedin_link: "test"
        }
	},
	"andyzheng": {
		teammate_profile: {
            name: "Andy Zheng",
            role: "Research Programmer",
            study: "Computer Science and Innovation",
            img_src: profilePlaceholder
        },
        background_description: `Lorem ipsum dolor sit amet, consectetur adipiscing elit. 
                                Nullam vulputate orci in mattis accumsan. Etiam fermentum hendrerit porta. Nam malesuada mauris enim, sed rhoncus nibh aliquet eu. Nunc vestibulum tempus mattis. Etiam consectetur turpis ultricies lectus dictum feugiat. Proin euismod id nisi tempor placerat. Phasellus eu faucibus massa, ut interdum orci. Duis sed tincidunt eros. Integer accumsan egestas facilisis. Ut porta mollis nisi, vel tincidunt nisi imperdiet vel. In imperdiet sodales nulla, non venenatis arcu pulvinar id. Quisque et metus quis libero facilisis iaculis et ullamcorper magna. Nam risus risus, blandit non consectetur quis, lobortis eget erat. In hac habitasse platea dictumst. Mauris vel erat suscipit, iaculis nibh vitae, tempus tortor. Quisque orci elit, pharetra et felis sit amet, suscipit consectetur diam.`,
        social_links: {
            github_link: "test",
            linkedin_link: "test"
        }
	},
	"deriso'malley": {
		teammate_profile: {
            name: "Deris O'Malley",
            role: "Research Programmer",
            study: "Computer Science and Innovation",
            img_src: profilePlaceholder
        },
        background_description: `Lorem ipsum dolor sit amet, consectetur adipiscing elit. 
                                Nullam vulputate orci in mattis accumsan. Etiam fermentum hendrerit porta. Nam malesuada mauris enim, sed rhoncus nibh aliquet eu. Nunc vestibulum tempus mattis. Etiam consectetur turpis ultricies lectus dictum feugiat. Proin euismod id nisi tempor placerat. Phasellus eu faucibus massa, ut interdum orci. Duis sed tincidunt eros. Integer accumsan egestas facilisis. Ut porta mollis nisi, vel tincidunt nisi imperdiet vel. In imperdiet sodales nulla, non venenatis arcu pulvinar id. Quisque et metus quis libero facilisis iaculis et ullamcorper magna. Nam risus risus, blandit non consectetur quis, lobortis eget erat. In hac habitasse platea dictumst. Mauris vel erat suscipit, iaculis nibh vitae, tempus tortor. Quisque orci elit, pharetra et felis sit amet, suscipit consectetur diam.`,
        social_links: {
            github_link: "test",
            linkedin_link: "test"
        }
	},
}