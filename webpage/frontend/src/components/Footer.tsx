import { NavLink } from 'react-router-dom';
import { handle_scroll_top } from '../util/helpers/scroll_top';
import { Linkedin, Github } from '@boxicons/react';

import './styles/Footer.css';

const Footer = () => {
    return (
        <div className="footer">
            <div className="bezier_footer"></div>
            <div className="footer_logo"></div>

            <div className='footer_line'></div>

            <div className="footer_links_container">
                <NavLink className="footer_link" onClick={handle_scroll_top} to={"/"}>Home</NavLink>
                <NavLink className="footer_link" onClick={handle_scroll_top} to={"/info"}>Background</NavLink>
                <NavLink className="footer_link" onClick={handle_scroll_top} to={"/results"}>Results</NavLink>
            </div>

            <div className='footer_line'></div>

            <div className="socials_container">
                <a href='#' className="social_button"><Linkedin /></a>
                <a href='#' className="social_button"><Github/></a>
            </div>
        </div>
    )
}

export default Footer;