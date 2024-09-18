from locust import HttpUser, TaskSet, SequentialTaskSet, task,  between
import uuid
import os
import signal
import sys
import json

config_set = {
   'version' : 1,
   'base' : uuid.uuid4().hex,
    'pi' : [],
    'pa' : [],
 }

def extract_cell(input):
    return input.split("_")[0]

class PaymentsBehaviour(SequentialTaskSet):
    payment_id = None
    attempt_version = [None] * 1
    cell = None
    
    def gen(self):
        config_set['version'] += 1
        return (config_set['base'] + str(config_set['version']))
    @task(1)
    def intent_create(self):
        payment_id = self.gen()
        self.payment_id = payment_id
        response = self.client.get('/create/' + self.payment_id + '/kaps', name = "IntentCreate",allow_redirects=True)
        if response.status_code == 200:
            self.cell = extract_cell(json.loads(response.text)["pi"])
            config_set['pi'].append(payment_id)
    @task(1)
    def pay(self):
        for i in range(1):
            self.attempt_version[i] = self.payment_id + 'version' + str(i)
            response = self.client.get('/pay/' + self.payment_id + '/' + self.attempt_version[i], name = "AttemptCreate",allow_redirects=True, headers = {'x-region' : self.cell})
            if response.status_code == 200:
                self.cell = extract_cell(json.loads(response.text)["pa"])
                config_set['pa'].append(self.payment_id + ',' + self.attempt_version[i] + '\n')
    @task(1)
    def update_attempt(self):
        for i in range(len(self.attempt_version)):
            if self.attempt_version[i] is not None:
                response = self.client.get('/update_attempt/pay/' + self.attempt_version[i] + '/' + self.payment_id , name = "UpdateAttempt",allow_redirects=True, headers = {'x-region' : self.cell})
    
    @task(1)
    def update_intent(self):
        response = self.client.get('/update_intent/' + self.payment_id, name = "UpdateIntent",allow_redirects=True, headers = {'x-region' : self.cell})
    
    # @task(1)
    # def retrieve_attempt(self):
    #     response = self.client.get('/retrieve/payment_attempt/' + self.payment_id, name = "RetrieveAllAttempt", headers = {'x-region' : self.cell})
    
    @task(1)
    def retrieve_intent(self):
        response = self.client.get('/retrieve/payment_intent/' + self.payment_id, name = "RetrieveIntent",allow_redirects=True, headers = {'x-region' : self.cell})

def make():
    return type("MyClass", (PaymentsBehaviour,), {})
    

class WebsiteUser(HttpUser):
    MyClass = make()
    tasks = [MyClass]


def sig_term_handler(_signo, _stack_frame):
    print("waht tht fook")
    f1 = open('intent.out', 'a')
    f2 = open('attempt.out', 'a')
    f1.write('\n'.join(config_set['pi']))
    f2.write(''.join(config_set['pa']))
    sys.exit(0)

signal.signal(signal.SIGTERM, sig_term_handler)
signal.signal(signal.SIGINT, sig_term_handler)