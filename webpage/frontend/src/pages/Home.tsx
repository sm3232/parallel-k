import { useNavigate } from 'react-router-dom';
import { handle_scroll_top } from '../util/helpers/scroll_top';
import { team_info } from '../util/consts/profiles';

import '../styles/pages.css';
import '../styles/common_styles.css';
import '../styles/homepage.css';

const Home = () => {
	const navigate = useNavigate();

	const navigate_to_bio = (name: string) => {
		navigate(`/researcher-bio/${name.replace(/\s/g, '')}`);
	}

    return (
		<div className="page_container">
			<div className="top_section">
				<h1>Data Privacy + Deep Learning</h1>

				<div className='top_section_background'></div>
			</div>

			<div className='page_sub_section'>
				<h2>Our Team</h2>
				<div className="team_container">
					{team_info.map((teammate) => (
						<div className='teammate_card' key={teammate.name}>
							<img src={teammate.img_src} className='profile_image'/>

							<button onClick={() => {
								navigate_to_bio(teammate.name);
								handle_scroll_top();
							}} className='bio_navigate_button'>Bio</button>

							<div className="team_card_text_container">
								<p className='text teammate_name'>{teammate.name}</p>
								<p className="text">{teammate.role}</p>
								<p className="text">{teammate.study}</p>
							</div>
						</div>
					))}
				</div>	
			</div>
		</div>
    )
}

export default Home;