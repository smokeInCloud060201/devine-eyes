import { BrowserRouter, Route, Routes } from 'react-router-dom';
import PageLayout from './components/PageLayout';
import ServiceMap from './pages/ServiceMap';
import Dashboard from './pages/Dashboard';
import APM from './pages/APM';

function App() {
    return (
        <BrowserRouter>

            <Routes>
                <Route path="/" element={
                    <PageLayout>
                        <Dashboard />
                    </PageLayout>} />
                <Route path="/service-map" element={ <PageLayout><ServiceMap /></PageLayout>} />
                <Route path="/apm" element={<PageLayout><APM /></PageLayout>} />
                <Route path="/images" element={<PageLayout><div className="p-6">Images page coming soon...</div></PageLayout>} />
            </Routes>
        </BrowserRouter>
    );
}

export default App;
