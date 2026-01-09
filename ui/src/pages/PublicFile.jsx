import { useParams } from '@solidjs/router'
import { Show, createSignal, onMount } from 'solid-js'
import Box from '@suid/material/Box'
import Button from '@suid/material/Button'
import Paper from '@suid/material/Paper'
import Typography from '@suid/material/Typography'
import Container from '@suid/material/Container'
import DownloadIcon from '@suid/icons-material/Download'
import InsertDriveFileIcon from '@suid/icons-material/InsertDriveFile'
import Stack from '@suid/material/Stack'
import CircularProgress from '@suid/material/CircularProgress'

import { PublicAPI } from '../api'
import { API_BASE } from '../api/request'

const PublicFile = () => {
	const params = useParams()
	const [file, setFile] = createSignal(null)
	const [loading, setLoading] = createSignal(true)
	const [error, setError] = createSignal(null)

	onMount(async () => {
		try {
			setLoading(true)
			const fileInfo = await PublicAPI.getPublicFile(
				params.storageId,
				params.path
			)
			setFile(fileInfo)
		} catch (err) {
			console.error(err)
			setError('File not found or access denied.')
		} finally {
			setLoading(false)
		}
	})

	const formatSize = (bytes) => {
		if (bytes === 0) return '0 Bytes'
		const k = 1024
		const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB']
		const i = Math.floor(Math.log(bytes) / Math.log(k))
		return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
	}

    const getDownloadUrl = () => {
        return `${API_BASE}/public/files/${params.storageId}/download/${params.path}`
    }

	return (
		<Container maxWidth="sm" sx={{ mt: 8 }}>
			<Paper elevation={3} sx={{ p: 4, textAlign: 'center' }}>
				<Show
					when={!loading()}
					fallback={<CircularProgress />}
				>
					<Show when={!error()} fallback={<Typography color="error">{error()}</Typography>}>
						<Stack spacing={3} alignItems="center">
							<InsertDriveFileIcon sx={{ fontSize: 80, color: 'primary.main' }} />
							
							<Box>
								<Typography variant="h5" component="h1" gutterBottom>
									{params.path.split('/').pop()}
								</Typography>
								<Typography variant="body1" color="text.secondary">
									Size: {formatSize(file()?.size || 0)}
								</Typography>
							</Box>

							<Button
								variant="contained"
								size="large"
								startIcon={<DownloadIcon />}
                                href={getDownloadUrl()}
                                component="a"
                                target="_blank"
								sx={{ px: 4, py: 1.5 }}
							>
								Download
							</Button>
                            <Typography variant="caption" color="text.secondary">
                                Download is handled by your browser. You can close this tab after download starts.
                            </Typography>
						</Stack>
					</Show>
				</Show>
			</Paper>
		</Container>
	)
}

export default PublicFile
