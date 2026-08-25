import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";
import type { Profile } from "../util/consts/profiles";
import { profile_bios } from "../util/consts/profiles";

import './styles/BioModal.css';

const BioModal = () => {
    const navigate = useNavigate();
    const { name } = useParams();
    const [ teammateData, setTeammateData ] = useState<Profile | null>();

    useEffect(() => {
        if(name !== undefined) {
            setTeammateData(profile_bios[name.toLowerCase()]);
        }
    }, []);

    return (
        <div className="bio_modal">
            {teammateData && (
                <div className="teammate_profile_container">
                    <button onClick={() => navigate(-1)} className="return_button"></button>
                    <div className="profile_header">{teammateData.teammate_profile.name}</div>
                    
                    <div className="profile_extra">
                        <div className="text">Role: {teammateData.teammate_profile.role}</div>
                        <div className="text">Study: {teammateData.teammate_profile.study}</div>
                    </div>

                    <h3>About Me</h3>
                    <div className="text">{teammateData.background_description}</div>
                </div>
            )}
        </div>
    )
}

export default BioModal;