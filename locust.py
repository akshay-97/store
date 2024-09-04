from locust import HttpUser, TaskSet, SequentialTaskSet, task,  between
import uuid
import os

config_set = {
   'version' : 1,
   'base' : uuid.uuid4().hex
}

f1 = file.open('intent.out', 'a')
f2 = file.open('attempt.out', 'a')
`
class PaymentsBehaviour(SequentialTaskSet):
    payment_id = None
    attempt_version = [None] * 1
    
    def gen(self):
        config_set['version'] += 1
        return (config_set['base'] + str(config_set['version']))
    @task(1)
    def intent_create(self):
        payment_id = self.gen()
        self.payment_id = payment_id
        f1.write(self.payment_id)
        response = self.client.get('/create/' + self.payment_id + '/' + 'kaps', name = "IntentCreate")
    @task(1)
    def pay(self):
        for i in range(1):
            self.attempt_version[i] = self.payment_id + 'version' + str(i)
            f2.write(self.attempt_version[i] )
            response = self.client.get('/pay/' + self.payment_id + '/' + self.attempt_version[i], name = "AttemptCreate")
    @task(1)
    def update_attempt(self):
        for i in range(len(self.attempt_version)):
            if self.attempt_version[i] is not None:
                response = self.client.get('/update_attempt/pay/' + self.attempt_version[i] + '/' + self.payment_id , name = "UpdateAttempt")
    
    @task(1)
    def update_intent(self):
        response = self.client.get('/update_intent/' + self.payment_id, name = "UpdateIntent")
    
    # @task(1)
    # def retrieve_attempt(self):
    #     response = self.client.get('/retrieve/payment_attempt/' + self.payment_id, name = "RetrieveAllAttempt")
    
    @task(1)
    def retrieve_intent(self):
        response = self.client.get('/retrieve/payment_intent/' + self.payment_id, name = "RetrieveIntent")

def make():
    return type("MyClass", (PaymentsBehaviour,), {})
    

class WebsiteUser(HttpUser):
    MyClass = make()
    tasks = [MyClass]
