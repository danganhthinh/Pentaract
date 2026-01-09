import { useBeforeLeave, useParams } from '@solidjs/router'
import { Show, createSignal, mapArray, onCleanup, onMount } from 'solid-js'
import List from '@suid/material/List'
import Grid from '@suid/material/Grid'
import Stack from '@suid/material/Stack'
import Typography from '@suid/material/Typography'
import Divider from '@suid/material/Divider'
import Container from '@suid/material/Container'
import Paper from '@suid/material/Paper'
import Box from '@suid/material/Box'

import { PublicAPI } from '../api'
import FSListItem from '../components/FSListItem'

const PublicDownload = () => {
	/**
	 * @type {[import("solid-js").Accessor<import("../api").FSElement[]>, any]}
	 */
	const [fsLayer, setFsLayer] = createSignal([])
	const [loading, setLoading] = createSignal(true)
	const [error, setError] = createSignal(null)
	const params = useParams()
	const basePath = `/download/${params.storageId}`

	const fetchFSLayer = async (path = params.path || '') => {
		try {
			setLoading(true)
			setError(null)
			const fsLayerRes = await PublicAPI.getPublicFSLayer(
				params.storageId,
				path
			)

			if (path.length) {
				const parentPath = path.split('/').slice(0, -1).join('/')
				const backToParent = { is_file: false, name: '..', path: parentPath }
				fsLayerRes.splice(0, 0, backToParent)
			}

			setFsLayer(fsLayerRes)
		} catch (err) {
			console.error(err)
			setError('Failed to load files. Please check the storage ID.')
		} finally {
			setLoading(false)
		}
	}

	const reload = async () => {
		if (window.location.pathname.startsWith(basePath)) {
			await fetchFSLayer()
		}
	}

	onMount(() => {
		fetchFSLayer()
		window.addEventListener('popstate', reload, false)
	})

	onCleanup(() => window.removeEventListener('popstate', reload, false))

	useBeforeLeave(async (e) => {
		if (e.to.startsWith(basePath)) {
			let newPath = e.to.slice(basePath.length)

			if (newPath.startsWith('/')) {
				newPath = newPath.slice(1)
			}

			await fetchFSLayer(newPath)
		}
	})

	return (
		<Container maxWidth="md" sx={{ py: 4 }}>
			<Paper elevation={3} sx={{ p: 3 }}>
				<Stack spacing={3}>
					<Box>
						<Typography variant="h4" gutterBottom>
							Public File Download
						</Typography>
						<Typography variant="body2" color="text.secondary">
							Storage ID: {params.storageId}
						</Typography>
						<Show when={params.path}>
							<Typography variant="body2" color="text.secondary">
								Path: /{params.path}
							</Typography>
						</Show>
					</Box>

					<Divider />

					<Show
						when={!loading() && !error()}
						fallback={
							<Typography variant="body1" sx={{ textAlign: 'center', py: 4 }}>
								{loading() ? 'Loading...' : error()}
							</Typography>
						}
					>
						<Grid>
							<Show when={fsLayer().length} fallback={<>No files yet</>}>
								<List sx={{ minWidth: 320, maxWidth: 540, mx: 'auto' }}>
									<Divider />
									{mapArray(fsLayer, (fsElement) => (
										<>
											<FSListItem
												fsElement={fsElement}
												storageId={params.storageId}
												isPublic={true}
											/>
											<Divider />
										</>
									))}
								</List>
							</Show>
						</Grid>
					</Show>
				</Stack>
			</Paper>
		</Container>
	)
}

export default PublicDownload
