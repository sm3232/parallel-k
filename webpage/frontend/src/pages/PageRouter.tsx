import { Routes, Route } from "react-router-dom";
import HomeInfo from "./Home";
import Results from "./Results";
import Research from "./Research";
import ResearchProfile from "./ResearchProfile";

const PageRouter = () => {
    return (
        <Routes>
            <Route path="/" element={<HomeInfo />} />
            <Route path="/info" element={<Research />} />
            <Route path="/results" element={<Results />} />
            <Route path="/researcher-bio/:name" element={<ResearchProfile />} />
        </Routes>
    )
}

export default PageRouter;