from __future__ import annotations

import json
import math
import os
from pathlib import Path
import sys
import unittest


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "python"))

import destiny  # noqa: E402
from destiny._backend import InMemoryBackend, NativeBackend  # noqa: E402


NATIVE_LIBRARY = os.environ.get("DESTINY_BEVY_COMPAT_LIBRARY")


@unittest.skipUnless(NATIVE_LIBRARY, "native differential gate requires a built library")
class NativeDifferentialTests(unittest.TestCase):
    def setUp(self) -> None:
        self.native = NativeBackend(NATIVE_LIBRARY)
        self.contract = InMemoryBackend()

    def tearDown(self) -> None:
        self.native.close()
        self.contract.close()

    def test_removal_resumes_after_each_mode_callback_and_final_cleanup(self) -> None:
        park = destiny.Ballpark(_backend=self.native)
        observations = []
        handlers = []
        target = park.AddBall(10, 1, 1, 1, False, False, False, False, False,
                              0, 0, 0, 0, 0, 0, 1, 1)

        class Follower(destiny.Ball):
            def DoModeChange(self, old, new):
                if old == destiny.DSTBALL_FOLLOW and new == destiny.DSTBALL_STOP:
                    observations.append((self.id, self.mode, self.followId, self.followRange,
                                         target.isMoribund, park.HasBall(target.id),
                                         [(ball.id, ball.followId) for ball in handlers]))

        class Target(destiny.Ball):
            def DoFinalCleanup(self):
                observations.append(("cleanup", self.isMoribund, park.HasBall(self.id),
                                     tuple(ball.mode for ball in handlers)))

        target_handler = Target(target)
        try:
            for ball_id in (1, 2):
                ball = park.AddBall(ball_id, 1, 1, 1, False, False, False, False, False,
                                    0, 0, 0, 0, 0, 0, 1, 1)
                handlers.append(Follower(ball))
                park.FollowBall(ball_id, target.id)
            park.RemoveBall(target.id)
            self.assertEqual(len(observations), 3)
            first, second, cleanup = observations
            for entry in (first, second):
                self.assertEqual(entry[1:6], (destiny.DSTBALL_FOLLOW, 0, 0.0, False, True))
            self.assertEqual(sorted(value for _, value in first[6]), [0, target.id])
            self.assertEqual([value for _, value in second[6]], [0, 0])
            self.assertEqual(cleanup, ("cleanup", True, True, (destiny.DSTBALL_STOP,) * 2))
            self.assertFalse(park.HasBall(target.id))
            self.assertIsNone(target_handler.ballpark)
        finally:
            park.close()

    def test_removal_rechecks_followers_after_reentrant_retarget(self) -> None:
        park = destiny.Ballpark(_backend=self.native)
        handlers = []
        stopped = []
        retargeting = [False]
        events = []
        class Follower(destiny.Ball):
            def DoModeChange(self, old, new):
                if old == destiny.DSTBALL_FOLLOW and new == destiny.DSTBALL_STOP:
                    events.append((self.id, retargeting[0], self.followId))
                    if retargeting[0]: return
                    stopped.append(self.id)
                    if len(stopped) == 1:
                        other = next(ball for ball in handlers if ball.id != self.id)
                        retargeting[0] = True
                        try: park.FollowBall(other.id, 20, 7)
                        finally: retargeting[0] = False
        try:
            for ball_id in (10, 20, 1, 2):
                ball = park.AddBall(ball_id, 1, 1, 1, False, False, False, False, False,
                                    0, 0, 0, 0, 0, 0, 1, 1)
                if ball_id in (1, 2): handlers.append(Follower(ball))
            for ball in handlers: park.FollowBall(ball.id, 10)
            park.RemoveBall(10)
            self.assertEqual(len(stopped), 1)
            other = next(ball for ball in handlers if ball.id != stopped[0])
            self.assertEqual(events, [(stopped[0], False, 0), (other.id, True, 0)])
            self.assertEqual((other.mode, other.followId, other.followRange),
                             (destiny.DSTBALL_FOLLOW, 20, 7.0))
            self.assertEqual(park.GetFollowers(20), [other.id])
        finally:
            park.close()

    def test_reentrant_follow_preserves_source_stale_registry_until_removal(self) -> None:
        park = destiny.Ballpark(_backend=self.native)
        class Handler(destiny.Ball):
            redirected = False
            def DoModeChange(self, old, new):
                if new == destiny.DSTBALL_FOLLOW and not self.redirected:
                    self.redirected = True
                    park.FollowBall(self.id, 3, 7)
        try:
            for ball_id in (1,2,3):
                ball = park.AddBall(ball_id,1,1,1,False,False,False,False,False,
                                    0,0,0,0,0,0,1,1)
                if ball_id == 1: handler = Handler(ball)
            park.FollowBall(1,2,5)
            self.assertEqual((handler.followId,handler.followRange),(3,7.0))
            # Follow inserts into its captured target after the mode callback.
            # The source registry can therefore retain both entries.
            self.assertEqual((park.GetFollowers(2),park.GetFollowers(3)),([1],[1]))
            park.RemoveBall(2)
            self.assertEqual((handler.mode,handler.followId,handler.followRange),
                             (destiny.DSTBALL_FOLLOW,3,7.0))
            self.assertEqual(park.GetFollowers(3),[1])
            park.Stop(1)
            self.assertEqual(park.GetFollowers(3),[])
        finally: park.close()

    def test_formation_shrink_notifies_follower_before_publishing_leader_definition(self) -> None:
        # A derived callback scenario with the source case's 4 -> 1 slot
        # transition; exact original offsets are exercised by the catalog run.
        formations=[("one",[(0,0,0),(1,0,0),(2,0,0),(3,0,0)]),
                    ("two",[(0,0,0),(1,0,0),(2,0,0),(3,0,0)]),
                    ("three",[(0,0,0)])]
        park=destiny.Ballpark(_backend=self.native)
        events=[]
        class Handler(destiny.Ball):
            def DoModeChange(self,old,new):
                if old==destiny.DSTBALL_FORMATION and new==destiny.DSTBALL_STOP:
                    events.append((self.mode,self.followId,leader.formationID,park.GetFollowers(leader.id)))
        try:
            leader=park.AddBall(2,1,1,1,False,False,False,False,False,0,0,0,0,0,0,1,1)
            member=Handler(park.AddBall(1,1,1,1,False,False,False,False,False,0,0,0,0,0,0,1,1))
            park.LoadFormations(formations);park.SetBallFormation(2,1);park.FormationFollow(1,2,1)
            park.SetBallFormation(2,2)
            self.assertEqual(events,[(destiny.DSTBALL_FORMATION,0,1,[])])
            self.assertEqual((member.mode,leader.formationID),(destiny.DSTBALL_STOP,2))
        finally:park.close()

    def test_retained_removal_clear_and_reused_id_have_distinct_objects(self) -> None:
        park = destiny.Ballpark(_backend=self.native)
        try:
            old = park.AddBall(1, 1, 1, 1, False, False, False, False, False,
                               17, 0, 0, 0, 0, 0, 1, 1)
            alias = destiny.Ball(old)
            old_handle = old._handle
            park.RemoveBall(1)
            self.assertTrue(old.isMoribund)
            self.assertTrue(alias.isMoribund)
            self.assertEqual((old.id, old.x, alias.x), (1, 17, 17))
            self.assertNotIn(1, park.balls)
            new = park.AddBall(1, 1, 1, 1, False, False, False, False, False,
                               29, 0, 0, 0, 0, 0, 1, 1)
            self.assertNotEqual(old_handle, new._handle)
            self.assertFalse(new.isMoribund)
            self.assertEqual((old.x, new.x), (17, 29))
            with self.assertRaises(destiny.BackendCallError):
                old.x = 40
            self.assertEqual(new.x, 29)
            park.ClearAll()
            self.assertFalse(new.isMoribund)  # ClearAll does not call RemoveBall.
            self.assertEqual(new.x, 29)
            replacement = park.AddBall(1, 1, 1, 1, False, False, False, False, False,
                                       43, 0, 0, 0, 0, 0, 1, 1)
            self.assertEqual((old.x, new.x, replacement.x), (17, 29, 43))
        finally:
            park.close()

    def test_native_bubble_callback_sees_mutated_dictionaries_and_permits_reentry(self) -> None:
        from destiny._util.decometaclass import WrapBlueClass
        outputs=({}, {}, {}, {})
        native=self.native
        # Decorator construction uses the explicit native backend below.
        class Handler(destiny.Ballpark):
            def DoBubbleTransitions(self, transitions):
                self.observations.append((transitions,[dict(value) for value in outputs],self.GetBall(1).oldBubbleId))
        park=Handler(_backend=native)
        park.observations=[]
        try:
            ball=park.AddBall(1,1,0,0,False,False,False,True,False,1e10,1e10,1e10,0,0,0,1,1)
            park.InitializeBubbles()
            park.AdditionsAndDeletions(*outputs,[1])
            expected=[{1:[1]},{1:[]},{ball.newBubbleId:[1]},{ball.newBubbleId:[]}]
            self.assertEqual(list(outputs),expected)
            self.assertEqual(park.observations,[([(1,-1,ball.newBubbleId)],expected,ball.newBubbleId)])
            park.CopyBubbles()
            park.AdditionsAndDeletions(*outputs,[1])
            self.assertEqual(outputs[0],{1:ball.newBubbleId})
            self.assertEqual(outputs[1],{1:ball.newBubbleId})
            self.assertEqual(park.observations[-1][0],[])
            # The original decorated __init__ overrides the facade's initializer.
            class Decorated(WrapBlueClass("destiny.Ballpark")):
                def __init__(self): self.transitions=[]
            destiny.set_backend(native)
            decorated=Decorated()
            try:
                self.assertIn(decorated._handle,native._runtimes)
            finally: decorated.close()
        finally: park.close()

    def test_native_bubble_dictionary_alias_and_invalid_user_prefix(self) -> None:
        park=destiny.Ballpark(_backend=self.native)
        try:
            park.AddBall(0,1,0,0,False,False,False,False,False,0,0,0,0,0,0,1,1)
            park.InitializeBubbles()
            shared={99:[99]}
            park.AdditionsAndDeletions(shared,shared,shared,shared,[0,0])
            self.assertEqual(shared,{0:0})
            outputs=({99:[99]},{99:[99]},{99:[99]},{99:[99]})
            with self.assertRaises(TypeError): park.AdditionsAndDeletions(*outputs,[None])
            self.assertEqual(outputs,({}, {}, {}, {}))
        finally: park.close()

    def test_native_detached_construction_alias_lease_and_cleanup(self) -> None:
        import gc
        before=set(self.native._runtimes)
        ball=destiny.Ball(_backend=self.native)
        client=destiny.ClientBall(_backend=self.native)
        alias=destiny.Ball(ball)
        self.assertIsNone(ball.ballpark)
        self.assertIsNone(client.ballpark)
        self.assertEqual((ball.mass,ball.radius,ball.mode,ball.followRange),(0,0,destiny.DSTBALL_STOP,10))
        self.assertFalse(ball.isMoribund)
        self.assertEqual(ball._handle,alias._handle)
        self.assertEqual(client.centerDist,-1)
        self.assertEqual(client.GetRotatedVector([1,2,3]),[1,2,3])
        self.assertEqual(ball.ReserveFormationSlot(),-1)
        self.assertEqual(len(ball._park.balls),0)
        arena=ball._park
        del ball
        self.assertEqual(alias.x,1e10)
        self.assertIn(arena._handle,self.native._runtimes)
        del alias,arena,client
        gc.collect()
        self.assertEqual(set(self.native._runtimes),before)

    def test_native_pending_removal_restore_then_clear_keeps_retained_moribund_state(self) -> None:
        park=destiny.Ballpark(_backend=self.native)
        try:
            ball=park.AddBall(1,1,1,1,False,False,False,False,False,0,0,0,0,0,0,1,1)
            park.RemoveBall(1,100)
            handle=ball._handle
            stamp,payload=park.CaptureFullState()
            from io import BytesIO
            park.ReadFullStateFromStream(BytesIO(payload), _restart_driver=False)
            self.assertEqual(ball._handle,handle)
            self.assertTrue(ball.isMoribund)
            park.ClearAll()
            self.assertTrue(ball.isMoribund)
        finally: park.close()

    def test_existing_native_park_observes_orientation_setting_revision_and_reset(self) -> None:
        park=destiny.Ballpark(_backend=self.native)
        try:
            configuration=destiny.settings.GetDefault()
            configuration.useDynamicalOrientation=True
            configuration.disableDynamicalOrientationForMissiles=True
            destiny.settings.Apply(configuration)
            ball=park.AddBall(1,1,1,1,False,False,False,False,False,0,0,0,0,0,0,1,1,2,0.75)
            self.assertEqual(ball.maxAngularVelocity,2)
            stamp,payload=park.CaptureFullState()
            state=json.loads(payload)
            self.assertTrue(state['park']['use_dynamical_orientation'])
            self.assertTrue(state['park']['disable_dynamical_orientation_for_missiles'])
            destiny.settings.Reset()
            park.AddBall(2,1,1,1,False,False,False,False,False,0,0,0,0,0,0,1,1)
            _,payload=park.CaptureFullState()
            self.assertFalse(json.loads(payload)['park']['use_dynamical_orientation'])
        finally:
            destiny.settings.Reset()
            park.close()

    def test_automatic_native_driver_does_not_retain_an_unowned_park_between_ticks(self) -> None:
        import gc
        import weakref
        from threading import Event
        from unittest.mock import patch
        import destiny._facade as facade
        waiting=Event()
        class ControlledStop(Event):
            def wait(self, timeout=None):
                waiting.set()
                return super().wait(timeout=10)
        park=destiny.Ballpark(_backend=self.native)
        handle=park._handle
        owner=weakref.ref(park)
        with patch.object(facade,"Event",ControlledStop):
            park.Start()
        worker=park._driver_thread
        self.assertTrue(waiting.wait(5))
        del park
        gc.collect()
        worker.join(timeout=5)
        self.assertIsNone(owner())
        self.assertFalse(worker.is_alive())
        self.assertNotIn(handle,self.native._runtimes)

    def test_native_callback_reentry_does_not_invert_driver_and_backend_locks(self) -> None:
        from threading import Event, Thread
        from unittest.mock import patch
        park = destiny.Ballpark(_backend=self.native)
        waiting = Event()
        completed = Event()
        errors = []
        original_invoke = self.native.invoke

        def competing_invoke(title, *args, **kwargs):
            if title == "dbc.compat.Ballpark.HasBall":
                waiting.set()  # The competing facade has reached the backend.
            return original_invoke(title, *args, **kwargs)

        def competitor():
            try:
                self.assertTrue(park.HasBall(1))
            except Exception as error:
                errors.append(error)
            finally:
                completed.set()

        try:
            ball = park.AddBall(1, 1, 0, 0, False, False, False, False, False,
                                0, 0, 0, 0, 0, 0, 1, 1)
            # This is the lock held by an executing native callback. Event
            # barriers control the contender; no timing sleep selects a race.
            with patch.object(self.native, "invoke", side_effect=competing_invoke):
                with self.native.ball_binding_session():
                    worker = Thread(target=competitor, daemon=True)
                    worker.start()
                    self.assertTrue(waiting.wait(5))
                    acquired = park._driver_lock.acquire(timeout=5)
                    self.assertTrue(acquired, "backend waiter retained the driver lock")
                    if acquired:
                        park._driver_lock.release()
                    self.assertEqual(park.GetBall(1).id, ball.id)
                self.assertTrue(completed.wait(5))
                worker.join(timeout=5)
                self.assertFalse(worker.is_alive())
            self.assertFalse(errors)
        finally:
            park.close()

    @staticmethod
    def run_scenario(backend):
        park = destiny.Ballpark(_backend=backend)
        try:
            first = park.AddBall(
                1, 5.0, 1.0, 100.0, True, False, True, True, False,
                0.0, 0.0, 0.0, 10.0, 0.0, 0.0, 0.5, 1.0,
            )
            park.AddBall(
                2, 5.0, 1.0, 100.0, True, False, True, False, False,
                5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 1.0,
            )
            park.tickInterval = 100.0
            park.friction = 5.0
            park.Evolve()
            capture_time, capture = park.CaptureFullState()
            capture_payload = json.loads(capture)
            return {
                "current_time": park.currentTime,
                "position": first.x,
                "velocity": first.vx,
                "range": park.GetBallIdsInRange(1, 10.0),
                "box": tuple(park.GetBoxCenter(2, 1.0, -1.0, 0.0)),
                "membership": park.GetBubbleMembership(),
                "capture_time": capture_time,
                "capture_ids": [row["id"] for row in capture_payload["balls"]],
            }
        finally:
            park.close()

    def test_supported_motion_query_and_geometry_results_match(self) -> None:
        native = self.run_scenario(self.native)
        contract = self.run_scenario(self.contract)
        self.assertEqual(native["current_time"], contract["current_time"])
        self.assertEqual(native["range"], contract["range"])
        self.assertEqual(native["box"], contract["box"])
        self.assertEqual(native["membership"], contract["membership"])
        self.assertEqual(native["capture_time"], contract["capture_time"])
        self.assertEqual(native["capture_ids"], contract["capture_ids"])
        self.assertTrue(math.isclose(native["position"], contract["position"], rel_tol=1e-10))
        self.assertTrue(math.isclose(native["velocity"], contract["velocity"], rel_tol=1e-10))

    def test_warp_callback_reentry_observes_old_mode_and_rejects_create_and_repair(self) -> None:
        park = destiny.Ballpark(_backend=self.native)
        park.isMaster = True
        first = park.AddBall(-1, 1, 1, 1, True, False, True, True, False,
                             0, 0, 0, 0, 0, 0, 1, 1)

        class Handler(destiny.Ball):
            def __init__(self, ball):
                super().__init__(ball)
                self.observations = []
                self.errors = []

            def DoModeChange(self, old, new):
                if old == destiny.DSTBALL_WARP and new == destiny.DSTBALL_STOP:
                    self.observations.append((old, new, self.mode, park.currentTime, tuple(park.balls)))
                    for ball_id in (self.id, 2):
                        try:
                            park.AddBall(ball_id, 99, 5, 6, True, False, True, True, False,
                                         1, 2, 3, 4, 5, 6, 1, 1)
                        except destiny.BackendCallError as error:
                            self.errors.append((ball_id, error.code))

        handler = Handler(first)
        # A later base-class projection must not replace the handler wrapper.
        projected = park.GetBall(first.id)
        self.assertEqual(projected.id, first.id)
        try:
            park.WarpTo(first.id, 100000.0, 200000.0, 300000.0)
            # A failure bound for this concrete source scenario, not a proof
            # scope or truncation of its loop. The original runner also runs
            # the unchanged unbounded while-not-warp_done scenario.
            for _ in range(10_000):
                park.Evolve()
                if handler.observations:
                    break
            self.assertEqual(len(handler.observations), 1)
            old, new, observed_mode, callback_time, ids = handler.observations[0]
            self.assertEqual((old, new, observed_mode), (3, 2, 3))
            self.assertEqual(callback_time + 1, park.currentTime)
            self.assertEqual(ids, (first.id,))
            self.assertEqual(tuple(park.balls), (first.id,))
            self.assertEqual(handler.errors, [(first.id, "invalid_request"), (2, "invalid_request")])
            self.assertEqual(first.mode, destiny.DSTBALL_STOP)
            self.assertEqual((first.mass, first.radius), (1.0, 1.0))
            park.AddBall(2, 1, 1, 1, True, False, True, True, False,
                         0, 0, 0, 0, 0, 0, 1, 1)
            self.assertTrue(park.HasBall(2))
        finally:
            park.close()

    def test_reentrant_close_keeps_the_native_enclosing_call_alive(self) -> None:
        park = destiny.Ballpark(_backend=self.native)
        first = park.AddBall(1, 1, 1, 1, True, False, True, True, False,
                             0, 0, 0, 0, 0, 0, 1, 1)

        class Handler(destiny.Ball):
            def DoModeChange(self, old, new):
                if new == destiny.DSTBALL_WARP:
                    park.close()

        handler = Handler(first)
        park.WarpTo(handler.id, 100000.0, 200000.0, 300000.0)
        self.assertTrue(park._closed)
        self.assertNotIn(park._handle, self.native._runtimes)

    def test_proximity_callback_removal_suppresses_later_callbacks_and_preserves_tick_phase(self) -> None:
        park = destiny.Ballpark(_backend=self.native)
        owner = park.AddBall(3, 1, 0, 0, False, False, False, False, False,
                             0, 0, 0, 0, 0, 0, 1, 0)

        class Handler(destiny.Ball):
            def __init__(self, ball):
                super().__init__(ball)
                self.observed = []

            def DoProximity(self, other, entered):
                self.observed.append((other, entered, park.currentTime))
                park.RemoveProximitySensor(self.id)
                # Creation remains forbidden through this later callback phase.
                try:
                    park.AddBall(6, 1, 0, 0, False, False, False, False, False,
                                 0, 0, 0, 0, 0, 0, 1, 0)
                except destiny.BackendCallError:
                    pass

        handler = Handler(owner)
        try:
            owner.AddProximitySensor(10, 0, False, False)
            for ball_id in (4, 5):
                park.AddBall(ball_id, 1, 0, 0, True, False, False, False, False,
                             0, 0, 0, 0, 0, 0, 1, 0)
            park.Evolve()
            self.assertEqual(len(handler.observed), 1)
            self.assertIn(handler.observed[0][0], (4, 5))
            self.assertEqual(handler.observed[0][1:], (1, 0))
            self.assertEqual(park.currentTime, 1)
            self.assertFalse(park.HasBall(6))
            park.Evolve()
            self.assertEqual(len(handler.observed), 1)
        finally:
            park.close()

    def test_native_restore_preserves_motion_notification_and_target_state(self) -> None:
        from io import BytesIO
        park = destiny.Ballpark(_backend=self.native)
        try:
            first = park.AddBall(1, 1, 1, 100, True, False, True, True, False,
                                 0, 0, 0, 0, 0, 0, 1, 1)
            park.AddBall(2, 1, 1, 100, False, False, True, True, False,
                         1000, 0, 0, 0, 0, 0, 1, 1)
            park.Orbit(1, 2, 10)
            park.SetNotificationRange(1, 20)
            park.SetTargetTracking(1, 2, 50)
            for _ in range(3):
                park.Evolve()
            stamp, snapshot = park.CaptureFullState()
            captured = json.loads(snapshot)
            row = next(row for row in captured["balls"] if row["id"] == 1)
            self.assertEqual(row["motion"]["mode"], 4)
            self.assertEqual(row["motion"]["follow_id"], 2)
            self.assertEqual(row["notifications"]["target"], 2)
            park.Evolve()
            expected = (first.x, first.y, first.z, first.vx, first.vy, first.vz)
            park.Stop(1)
            park.RemoveBall(2)
            park.ReadFullStateFromStream(BytesIO(snapshot), 1, _restart_driver=False)
            self.assertEqual(park.currentTime, stamp)
            self.assertTrue(park.HasBall(2))
            self.assertEqual(first.mode, 4)
            park.Evolve()
            self.assertEqual((first.x, first.y, first.z, first.vx, first.vy, first.vz), expected)
        finally:
            park.close()

    def test_proximity_resumes_with_live_geometry_after_each_synchronous_callback(self) -> None:
        park = destiny.Ballpark(_backend=self.native)
        owner = park.AddBall(1, 0, 0, 0, False, False, False, False, False,
                             0, 0, 0, 0, 0, 0, 1, 0)
        for ball_id in (2, 3):
            park.AddBall(ball_id, 0, 0, 0, True, False, False, False, False,
                         5, 0, 0, 0, 0, 0, 1, 0)

        class Handler(destiny.Ball):
            def __init__(self, ball):
                super().__init__(ball)
                self.observed = []

            def DoProximity(self, other, entered):
                self.observed.append((other, entered))
                if other == 2:
                    park.SetBallPosition(3, 100, 0, 0)

        handler = Handler(owner)
        try:
            owner.AddProximitySensor(10, 0, False, False)
            park.Evolve()
            self.assertEqual(handler.observed, [(2, 1)])
            self.assertEqual(park.GetBall(3).x, 100)
            state = json.loads(park.CaptureFullState()[1])
            self.assertEqual(state["balls"][0]["sensors"][0]["members"], [2])
        finally:
            park.close()

    def test_range_callback_changes_tracking_and_sensor_stages_in_the_same_tick(self) -> None:
        park = destiny.Ballpark(_backend=self.native)
        owner = park.AddBall(1, 1, 0, 0, True, False, False, False, False,
                             0, 0, 0, 0, 0, 0, 1, 0)
        park.AddBall(2, 1, 0, 0, True, False, False, False, False,
                     0, 0, 0, 0, 0, 0, 1, 0)

        class Handler(destiny.Ball):
            def __init__(self, ball):
                super().__init__(ball)
                self.observed = []

            def DoRange(self, entered):
                self.observed.append(("range", entered))
                park.SetTargetTracking(self.id, 0, 0)
                park.RemoveProximitySensor(self.id)

            def DoTargetTracking(self, entered):
                self.observed.append(("tracking", entered))

            def DoProximity(self, other, entered):
                self.observed.append(("sensor", other, entered))

        handler = Handler(owner)
        try:
            park.GotoPoint(1, 0, 0, 0)
            park.SetNotificationRange(1, 10)
            park.SetTargetTracking(1, 2, 10)
            owner.AddProximitySensor(10, 0, False, False)
            park.Evolve()
            self.assertEqual(handler.observed, [("range", 1)])
            self.assertEqual(park.currentTime, 1)
        finally:
            park.close()

    def test_sensor_replacement_preserves_the_original_unshuffled_elapsed_clock(self) -> None:
        park = destiny.Ballpark(_backend=self.native)
        owner = park.AddBall(1, 1, 0, 0, False, False, False, False, False,
                             0, 0, 0, 0, 0, 0, 1, 0)
        park.AddBall(2, 1, 0, 0, True, False, False, False, False,
                     5, 0, 0, 0, 0, 0, 1, 0)

        class Handler(destiny.Ball):
            def __init__(self, ball):
                super().__init__(ball)
                self.observed = []

            def DoProximity(self, other, entered):
                self.observed.append((other, entered))

        handler = Handler(owner)
        try:
            owner.AddProximitySensor(10, 2, False, False)
            park.Evolve()
            self.assertEqual(handler.observed, [])
            owner.AddProximitySensor(11, 2, False, False)
            state = json.loads(park.CaptureFullState()[1])
            self.assertEqual(state["balls"][0]["sensors"][0]["elapsed"], 1)
            park.Evolve()
            self.assertEqual(handler.observed, [(2, 1)])
        finally:
            park.close()

    def test_native_new_orbit_snapshot_restores_controller_and_tick_environment(self) -> None:
        from io import BytesIO
        source_backend = NativeBackend(NATIVE_LIBRARY, {"use_new_orbit": True})
        source = destiny.Ballpark(_backend=source_backend)
        restored = destiny.Ballpark(_backend=self.native)
        try:
            source.tickInterval = 100
            source.friction = 350000
            for ball_id, x in ((1, 0), (2, 104)):
                source.AddBall(ball_id, 13000000, 2, 10, True, False, True, True, False,
                               x, 0, 0, 0, 0, 1 if ball_id == 2 else 0, 0.9, 0.95)
            source.Orbit(2, 1, 100)
            source.Evolve()
            stamp, payload = source.CaptureFullState()
            decoded = json.loads(payload)
            self.assertTrue(decoded["park"]["use_new_orbit"])
            self.assertEqual(decoded["park"]["tick_interval_ms"], 100)
            source.Evolve()
            expected_ball = source.GetBall(2)
            expected = tuple(getattr(expected_ball, name) for name in ("x", "y", "z", "vx", "vy", "vz"))
            restored.ReadFullStateFromStream(BytesIO(payload))
            self.assertEqual(restored.currentTime, stamp)
            self.assertEqual(restored.tickInterval, 100)
            self.assertEqual(restored.friction, 350000)
            restored.Evolve()
            actual_ball = restored.GetBall(2)
            self.assertEqual(tuple(getattr(actual_ball, name) for name in ("x", "y", "z", "vx", "vy", "vz")), expected)
        finally:
            source.close(); restored.close(); source_backend.close()

    def test_native_existing_park_observes_validated_settings_revisions(self) -> None:
        park = destiny.Ballpark(_backend=self.native)
        try:
            configuration = destiny.settings.GetDefault()
            configuration.collisionMaxIterations = 32
            configuration.useIterativeCollision = True
            configuration.useNewOrbit = True
            destiny.settings.Apply(configuration)
            state = json.loads(park.CaptureFullState()[1])["park"]
            self.assertEqual(state["collision_substeps"], 32)
            self.assertTrue(state["use_iterative_collision"])
            self.assertTrue(state["use_new_orbit"])
            configuration.collisionMaxIterations = 0
            with self.assertRaises(ValueError):
                destiny.settings.Apply(configuration)
            self.assertEqual(json.loads(park.CaptureFullState()[1])["park"], state)
            destiny.settings.Reset()
            state = json.loads(park.CaptureFullState()[1])["park"]
            self.assertEqual(state["collision_substeps"], 20)
            self.assertFalse(state["use_iterative_collision"])
            self.assertFalse(state["use_new_orbit"])
        finally:
            park.close()
            destiny.settings.Reset()

    @staticmethod
    def native_ticker(park):
        from destiny.net.client import Ticker, TickErrorHandlerInterface, ClientTickerInterface

        class Errors(TickErrorHandlerInterface):
            def __init__(self): self.fatal = False; self.recoverable = False
            def on_fatal_desync(self): self.fatal = True
            def on_recoverable_desync(self): self.recoverable = True

        class Interface(ClientTickerInterface):
            def on_set_state(self): pass
            def on_ballpark_local_action(self, func_name, args): pass
            def should_log_actions(self): return True
            def get_ball_destruction_delay(self, ball, is_terminal=False): return 0
            def get_ball_destruction_delays(self, ball_ids, is_release=False): return dict.fromkeys(ball_ids, 0)
            def clean_up_after_ball_removal(self, ball_id, ball, is_terminal=False): pass
            def clean_up_after_multiple_ball_removal(self, ball_ids, is_release=False): pass

        errors = Errors()
        ticker = Ticker(errors, Interface())
        ticker.set_ballpark(park)
        return ticker, errors

    def test_native_client_ticker_rewinds_two_ticks_and_restores_removed_entities(self) -> None:

        source = destiny.Ballpark(_backend=self.native)
        park = destiny.Ballpark(_backend=self.native)
        ticker, errors = self.native_ticker(park)
        try:
            for ball_id, x in ((1, 0), (2, 1000), (3, -1000)):
                source.AddBall(ball_id, 1, 1, 1, True, False, True, True, False,
                               x, 0, 0, 0, 0, 0, 1, 1)
            source.Orbit(2, 1)
            source.GotoPoint(3, 0, 0, 0)
            source.GotoDirection(1, 0, 1, 0)
            for _ in range(10): source.Evolve()
            stamp, payload = source.CaptureFullState()
            ticker.update([[stamp, ("SetState", (payload, 1))]], False)
            ticker.do_pre_tick()
            self.assertFalse(errors.fatal)
            park._parent_Evolve(); ticker.do_post_tick(10)
            park._parent_Evolve(); ticker.do_post_tick(11)
            park.RemoveBall(1)
            self.assertNotIn(1, park.balls)
            park._parent_Evolve(); ticker.do_post_tick(12)
            self.assertEqual(park.currentTime, 13)
            self.assertTrue(ticker.synchronize_to_simulation_time(11))
            self.assertEqual(park.currentTime, 11)
            self.assertIn(1, park.balls)
            self.assertEqual(park.GetBall(2).mode, destiny.DSTBALL_ORBIT)
            self.assertFalse(errors.fatal)
        finally:
            source.close(); park.close()

    def test_native_ticker_history_keeps_queued_updates_and_uses_original_snapshot_interval(self) -> None:
        park = destiny.Ballpark(_backend=self.native)
        ticker, errors = self.native_ticker(park)
        try:
            ticker.store_state()
            self.assertEqual(ticker._states, [])
            park.Start()
            self.assertIsNone(park._driver_thread)
            ticker.store_state()
            self.assertEqual(ticker._last_stamp, 0)
            queued = [(20, ("Stop", (1,)))]
            ticker.update(queued, False)
            ticker.clear_states()
            self.assertEqual(ticker._history, [[queued, False]])
            for stamp in range(11):
                park._parent_Evolve()
                ticker.do_post_tick(stamp)
            self.assertEqual(ticker._states, [])
            self.assertEqual(ticker._last_stamp, 0)
            park._parent_Evolve()
            ticker.do_post_tick(11)
            self.assertEqual([state[0] for state in ticker._states], [12])
            self.assertEqual(ticker._last_stamp, 12)
            ticker.clear_states()
            for _ in range(11):
                ticker.store_state()
                park._parent_Evolve()
            self.assertEqual([state[0] for state in ticker._states], [12, 18, 19, 20, 21, 22])
            self.assertFalse(errors.fatal)
        finally:
            park.close()

    def test_native_manual_scheduler_reservation_blocks_start_during_driver_join(self) -> None:
        from threading import Event, Thread, current_thread
        from unittest.mock import patch

        park = destiny.Ballpark(_backend=self.native)
        entered, resume = Event(), Event()
        errors = []
        owner = type("SchedulerOwner", (), {})()
        original_stop = destiny.Ballpark._stop_driver

        def gated_stop(instance, *args, **kwargs):
            if current_thread() is claimant:
                entered.set()
                if not resume.wait(2):
                    raise AssertionError("scheduler race coordination timed out")
            return original_stop(instance, *args, **kwargs)

        def claim():
            try:
                park._claim_manual_scheduler(owner)
            except BaseException as error:
                errors.append(error)

        claimant = Thread(target=claim)
        try:
            with patch.object(destiny.Ballpark, "_stop_driver", gated_stop):
                claimant.start()
                self.assertTrue(entered.wait(2))
                try:
                    park.Start()
                    self.assertIsNone(park._driver_thread)
                    self.assertIs(park._manual_scheduler_owner(), owner)
                finally:
                    resume.set()
                    claimant.join(2)
                self.assertFalse(claimant.is_alive())
                self.assertEqual(errors, [])
            self.assertIs(park._manual_scheduler_owner(), owner)
        finally:
            resume.set()
            claimant.join(2)
            park.close()

    def test_native_parent_alias_bypasses_subclass_evolve_override(self) -> None:
        class DerivedPark(destiny.Ballpark):
            def Evolve(self):
                self._override_calls += 1
                self._parent_Evolve()

        park = DerivedPark(_backend=self.native)
        try:
            park._override_calls = 0
            park.Evolve()
            self.assertEqual(park.currentTime, 1)
            self.assertEqual(park._override_calls, 1)
            park._parent_Evolve()
            self.assertEqual(park.currentTime, 2)
            self.assertEqual(park._override_calls, 1)
        finally:
            park.close()

    def test_native_adapted_late_arrival_preserves_original_tick_and_mode_assertions(self) -> None:
        from io import BytesIO

        # test_ticker.py::TestTicker::test_late_arrival and its exact 3-ball
        # setup, using native JSON/BytesIO in place of unavailable Blue streams.
        # This is adapted boundary evidence, not execution of unchanged Blue.
        source = destiny.Ballpark(_backend=self.native)
        park = destiny.Ballpark(_backend=self.native)
        ticker, errors = self.native_ticker(park)
        try:
            for id, x in [(1, 0.0), (2, 1000.0), (3, -1000.0)]:
                source.AddBall(id, 1.0, 1.0, 1.0, True, False, True, True, False,
                               x, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0)
            source.Orbit(2, 1)
            source.GotoPoint(3, 0.0, 0.0, 0.0)
            source.GotoDirection(1, 0.0, 1.0, 0.0)
            for _ in range(10):
                source.Evolve()
            stream = BytesIO()
            source.WriteFullStateToStream(stream)
            ticker.update([[0, ("SetState", (stream.getvalue(), 1))]], False)

            def tick(stamp):
                ticker.do_pre_tick()
                park._parent_Evolve()
                ticker.do_post_tick(stamp)

            tick(10)
            self.assertEqual(park.currentTime, 11)
            ball_two = park.balls[2]
            tick(11)
            tick(12)
            self.assertEqual(destiny.DSTBALL_ORBIT, ball_two.mode)
            ticker.update([[12, ("Stop", (2,))]], False)
            tick(13)
            self.assertEqual(destiny.DSTBALL_STOP, ball_two.mode)
            self.assertFalse(errors.fatal)
            self.assertFalse(errors.recoverable)
        finally:
            source.close()
            park.close()

    def test_native_mini_ball_identity_owner_and_restore_use_registered_body(self) -> None:
        from io import BytesIO
        park=destiny.Ballpark(_backend=self.native)
        try:
            parent=park.AddBall(1,13_000_000.0,2.0,10.0,False,False,False,False,False,
                                100.0,200.0,300.0,0.0,0.0,0.0,0.9,0.95)
            parent.AddMiniBall(1.0,2.0,3.0,10.0)
            mini=parent.miniBalls[0]
            first_id=mini.id
            self.assertEqual(first_id,-(1<<30)-1)
            self.assertEqual((mini.x,mini.y,mini.z),(1.0,2.0,3.0))
            child=park.GetBall(first_id)
            self.assertEqual((child.x,child.y,child.z),(101.0,202.0,303.0))
            self.assertFalse(child.isFree)
            self.assertTrue(child.isMassive)
            self.assertEqual(child.mode,destiny.DSTBALL_MINIBALL)
            stream=BytesIO();park.WriteFullStateToStream(stream)
            park.ClearAll()
            self.assertEqual(list(park.balls),[])
            park.ReadFullStateFromStream(BytesIO(stream.getvalue()))
            self.assertEqual(set(park.balls),{1,first_id})
            self.assertEqual(park.GetBall(1).miniBalls[0].id,first_id)
            self.assertEqual(park.GetBall(first_id).mode,destiny.DSTBALL_MINIBALL)
            self.assertEqual(park.GetBall(first_id).Agility,0.0)
            parent=park.GetBall(1)
            parent.AddMiniBall(4.0,5.0,6.0,2.0)
            self.assertEqual(parent.miniBalls[1].id,first_id-1)
            park.SetBallFree(1,True)
            self.assertEqual(set(park.balls),{1})
            park.SetBallFree(1,False)
            new_ids={parent.miniBalls[i].id for i in range(2)}
            self.assertEqual(new_ids,{first_id-2,first_id-3})
            self.assertEqual(set(park.balls),{1,*new_ids})
            park.RemoveBall(1)
            park.Evolve()
            self.assertEqual(list(park.balls),[])
        finally:
            park.close()

    def test_native_original_auto_identity_preserves_master_and_client_partitions(self) -> None:
        from io import BytesIO
        for master in (False,True):
            park=destiny.Ballpark(master,_backend=self.native)
            try:
                def add(id):
                    return park.AddBall(id,1.0,1.0,1.0,False,False,False,False,False,
                        0.0,0.0,0.0,0.0,0.0,0.0,1.0,1.0)
                zero=add(0)
                self.assertEqual(zero.id,-2 if master else 0)
                local=add(-1)
                self.assertEqual(local.id,-(1<<30)-1)
                stream=BytesIO();park.WriteFullStateToStream(stream)
                park.ClearAll();park.ReadFullStateFromStream(BytesIO(stream.getvalue()))
                self.assertEqual(add(-1).id,local.id-1)
                self.assertIn(local.id,park.balls)
            finally:
                park.close()



if __name__ == "__main__":
    unittest.main()
