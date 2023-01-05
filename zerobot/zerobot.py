#!/usr/bin/python -Bu

import server, asyncio, json
import openimc


class RobotHandler(server.WebSocketHandler):
	async def sendData(self):
		try:
			while not self.task.cancelled():
				self.write_messageJson({'jnt': openimc.get_joints()})
				await asyncio.sleep(0.04)
		except asyncio.CancelledError:
			pass

	def open(self):
		self.target = None
		self.speed  = 1
		self.task = asyncio.create_task(self.sendData())

	def on_close(self):
		self.task.cancel()

	def on_message(self, msg):
		msg = json.loads(msg)
		print(msg);
		if msg['cmd'] == 1:
			self.target = {
				'A': [270,  90, 1600, -90, 2000],
				'B': [180,  30,  800, -30, 1000],
				'C': [ 90, -30,  400,  30,  500],
				'D': [  0, -90,  100,  90,    0],
			}[msg['p']]
		if msg['cmd'] == 2:
			self.speed = msg['v']
		if self.target:
			openimc.move_joints(self.target, self.speed);



server.addDocument('', 'simulation.html')
server.addAjax('data', RobotHandler)

server.run(54321)
